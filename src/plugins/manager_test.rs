#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::Options;
    use crate::creds::{Credentials, parse_expression};
    use crate::plugins::plugin::{PayloadStrategy, Plugin};
    use crate::session::{Error, Loot, Session};
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Duration;

    // Mock plugin for testing
    struct MockPlugin {
        #[allow(dead_code)]
        name: String,
        setup_called: Arc<AtomicBool>,
        attempt_count: Arc<AtomicUsize>,
        fail_after: Option<usize>,
        return_loot: bool,
        single_payload: bool,
        override_payload: Option<crate::creds::Expression>,
        attempt_delay: Option<Duration>,
        setup_error: Option<String>,
        attempt_error: Option<String>,
    }

    impl MockPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                setup_called: Arc::new(AtomicBool::new(false)),
                attempt_count: Arc::new(AtomicUsize::new(0)),
                fail_after: None,
                return_loot: false,
                single_payload: false,
                override_payload: None,
                attempt_delay: None,
                setup_error: None,
                attempt_error: None,
            }
        }

        fn with_loot(mut self) -> Self {
            self.return_loot = true;
            self
        }

        fn with_single_payload(mut self) -> Self {
            self.single_payload = true;
            self
        }

        fn with_override_payload(mut self, expr: crate::creds::Expression) -> Self {
            self.override_payload = Some(expr);
            self
        }

        fn with_attempt_delay(mut self, delay: Duration) -> Self {
            self.attempt_delay = Some(delay);
            self
        }

        fn with_setup_error(mut self, error: String) -> Self {
            self.setup_error = Some(error);
            self
        }

        fn with_attempt_error(mut self, error: String) -> Self {
            self.attempt_error = Some(error);
            self
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn description(&self) -> &'static str {
            "Mock plugin for testing"
        }

        fn payload_strategy(&self) -> PayloadStrategy {
            if self.single_payload {
                PayloadStrategy::Single
            } else {
                PayloadStrategy::UsernamePassword
            }
        }

        fn override_payload(&self) -> Option<crate::creds::Expression> {
            self.override_payload.clone()
        }

        async fn setup(&mut self, _options: &Options) -> Result<(), Error> {
            self.setup_called.store(true, Ordering::Relaxed);
            if let Some(error) = &self.setup_error {
                return Err(error.clone());
            }
            Ok(())
        }

        async fn attempt(
            &self,
            creds: &Credentials,
            _timeout: Duration,
        ) -> Result<Option<Vec<Loot>>, Error> {
            let count = self.attempt_count.fetch_add(1, Ordering::Relaxed) + 1;

            if let Some(delay) = self.attempt_delay {
                tokio::time::sleep(delay).await;
            }

            if let Some(fail_after) = self.fail_after {
                if count > fail_after {
                    return Err("Failed after specified attempts".to_string());
                }
            }

            if let Some(error) = &self.attempt_error {
                return Err(error.clone());
            }

            if self.return_loot {
                Ok(Some(vec![Loot::new(
                    "mock",
                    &creds.target,
                    vec![
                        ("username".to_string(), creds.username.clone()),
                        ("password".to_string(), creds.password.clone()),
                    ],
                )]))
            } else {
                Ok(None)
            }
        }
    }

    // Helper function to create test options
    fn create_test_options(plugin_name: Option<String>) -> Options {
        let mut options = Options::default();
        options.plugin = plugin_name;
        options.target = Some("127.0.0.1:80".to_string());
        options.username = Some("admin".to_string());
        options.password = Some("password".to_string());
        options.concurrency = 2;
        options.timeout = 1000;
        options.retry_time = 100;
        options.retries = 3;
        options.quiet = true;
        options
    }

    fn create_inventory_with_plugin(plugin_name: &'static str, plugin: MockPlugin) -> Inventory {
        let mut inventory = Inventory::new();
        inventory.register(plugin_name, plugin);
        inventory
    }

    #[test]
    fn test_plugin_registration() {
        let mut inventory = Inventory::new();
        let plugin = MockPlugin::new("test_plugin_registration");

        inventory.register("test_plugin_registration", plugin);

        assert!(inventory.contains_key("test_plugin_registration"));
        assert_eq!(inventory.len(), 1);
    }

    #[test]
    fn test_multiple_plugin_registration() {
        let mut inventory = Inventory::new();

        inventory.register("plugin1", MockPlugin::new("plugin1"));
        inventory.register("plugin2", MockPlugin::new("plugin2"));
        inventory.register("plugin3", MockPlugin::new("plugin3"));

        assert_eq!(inventory.len(), 3);
        assert!(inventory.contains_key("plugin1"));
        assert!(inventory.contains_key("plugin2"));
        assert!(inventory.contains_key("plugin3"));
    }

    #[tokio::test]
    async fn test_plugin_setup_success() {
        *INVENTORY.lock().unwrap() = create_inventory_with_plugin(
            "test_plugin_setup_success",
            MockPlugin::new("test_plugin_setup_success"),
        );

        let options = create_test_options(Some("test_plugin_setup_success".to_string()));
        let result = setup(&options).await;

        match result {
            Ok(plugin_ref) => assert_eq!(plugin_ref.description(), "Mock plugin for testing"),
            Err(e) => {
                println!("error: {}", e);
                assert!(false);
            }
        }
    }

    #[tokio::test]
    async fn test_plugin_setup_with_error() {
        *INVENTORY.lock().unwrap() = create_inventory_with_plugin(
            "test_plugin_setup_with_error",
            MockPlugin::new("test_plugin_setup_with_error")
                .with_setup_error("Setup failed".to_string()),
        );

        let options = create_test_options(Some("test_plugin_setup_with_error".to_string()));
        let result = setup(&options).await;

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "Setup failed");
    }

    #[tokio::test]
    async fn test_plugin_not_found() {
        let inventory = Inventory::new();
        *INVENTORY.lock().unwrap() = inventory;

        let options = create_test_options(Some("nonexistent".to_string()));
        let result = setup(&options).await;

        assert!(result.is_err());
        assert!(result.err().unwrap().contains("is not a valid plugin name"));
    }

    #[tokio::test]
    async fn test_no_plugin_selected() {
        let options = create_test_options(None);
        let result = setup(&options).await;

        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "no plugin selected");
    }

    #[tokio::test]
    async fn test_worker_processes_credentials() {
        let plugin = MockPlugin::new("test_worker_processes_credentials").with_loot();
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let options = create_test_options(Some("test_worker_processes_credentials".to_string()));
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        // Send some credentials
        let creds = Credentials {
            target: "127.0.0.1:80".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        session.send_credentials(creds.clone()).await.unwrap();
        session.send_credentials(creds.clone()).await.unwrap();

        // Start worker
        let worker_session = session.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables, worker_session).await;
        });

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop the worker
        session.set_stop();

        // Wait for worker to finish
        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        // Note: can't check attempt count after boxing
        assert_eq!(session.get_done(), 2);
    }

    #[tokio::test]
    async fn test_worker_handles_errors_with_retry() {
        let plugin = MockPlugin::new("test_worker_handles_errors_with_retry")
            .with_attempt_error("Connection failed".to_string());
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let mut options =
            create_test_options(Some("test_worker_handles_errors_with_retry".to_string()));
        options.retries = 3;
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        let creds = Credentials {
            target: "127.0.0.1:80".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        session.send_credentials(creds).await.unwrap();

        let worker_session = session.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables.clone(), worker_session).await;
        });

        tokio::time::sleep(Duration::from_millis(500)).await;
        session.set_stop();

        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        // Should have attempted 3 times (retries)
        assert_eq!(session.get_errors(), 1);
    }

    #[tokio::test]
    async fn test_parallel_workers() {
        let plugin = MockPlugin::new("test_parallel_workers")
            .with_loot()
            .with_attempt_delay(Duration::from_millis(10));
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let mut options = create_test_options(Some("test_parallel_workers".to_string()));
        options.concurrency = 4;
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        // Send multiple credentials
        for i in 0..10 {
            let creds = Credentials {
                target: format!("127.0.0.1:{}", 8000 + i),
                username: format!("user{}", i),
                password: format!("pass{}", i),
            };
            session.send_credentials(creds).await.unwrap();
        }

        // Start multiple workers
        let mut handles = vec![];
        for _ in 0..4 {
            let worker_session = session.clone();
            let unreachables = unreachables.clone();
            let handle = tokio::spawn(async move {
                worker(plugin_ref, unreachables, worker_session).await;
            });
            handles.push(handle);
        }

        // Wait for processing
        tokio::time::sleep(Duration::from_millis(200)).await;
        session.set_stop();

        // Wait for all workers
        for handle in handles {
            tokio::time::timeout(Duration::from_secs(1), handle)
                .await
                .expect("Worker didn't finish in time")
                .expect("Worker panicked");
        }

        assert_eq!(session.get_done(), 10);
    }

    #[tokio::test]
    async fn test_worker_respects_stop_signal() {
        let plugin = MockPlugin::new("test_worker_respects_stop_signal")
            .with_attempt_delay(Duration::from_millis(100));
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let options = create_test_options(Some("test_worker_respects_stop_signal".to_string()));
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        // Send many credentials
        for i in 0..100 {
            let creds = Credentials {
                target: format!("127.0.0.1:{}", 8000 + i),
                username: "user".to_string(),
                password: "pass".to_string(),
            };
            session.send_credentials(creds).await.unwrap();
        }

        let worker_session = session.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables, worker_session).await;
        });

        // Let it process a few
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Stop early
        session.set_stop();

        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        // Should have processed less than all
        assert!(session.get_done() < 100);
    }

    #[tokio::test]
    async fn test_worker_with_jitter() {
        let plugin = MockPlugin::new("test_worker_with_jitter");
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let mut options = create_test_options(Some("test_worker_with_jitter".to_string()));
        options.jitter_min = 10;
        options.jitter_max = 50;
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        let creds = Credentials {
            target: "127.0.0.1:80".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        session.send_credentials(creds).await.unwrap();

        let start = std::time::Instant::now();

        let worker_session = session.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables, worker_session).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        session.set_stop();

        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        let elapsed = start.elapsed();

        // Should have added some jitter delay
        assert!(elapsed >= Duration::from_millis(10));
        assert_eq!(session.get_done(), 1);
    }

    #[tokio::test]
    async fn test_unreachable_targets_skipped() {
        let plugin = MockPlugin::new("test_unreachable_targets_skipped");
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let options = create_test_options(Some("test_unreachable_targets_skipped".to_string()));
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        // Mark a target as unreachable
        unreachables.insert(Arc::from("127.0.0.1:80"));

        let creds = Credentials {
            target: "127.0.0.1:80".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        session.send_credentials(creds).await.unwrap();

        let worker_session = session.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables, worker_session).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        session.set_stop();

        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        // Should not have attempted due to unreachable
        assert_eq!(session.get_done(), 1); // Still marked as done
    }

    #[tokio::test]
    async fn test_single_payload_strategy() {
        let test_payload = "test_payload".to_string();
        let plugin = MockPlugin::new("test_single_payload_strategy")
            .with_single_payload()
            .with_override_payload(parse_expression(Some(&test_payload)));
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_mut: &'static mut dyn Plugin = Box::leak(plugin_box);

        let mut inventory = Inventory::new();
        inventory.register(
            "test_single_payload_strategy",
            MockPlugin::new("test_single_payload_strategy")
                .with_single_payload()
                .with_override_payload(parse_expression(Some(&test_payload))),
        );
        *INVENTORY.lock().unwrap() = inventory;

        let options = create_test_options(Some("test_single_payload_strategy".to_string()));
        let session = Session::new_for_tests(options).unwrap();

        let result = run(plugin_mut, session.clone()).await;

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(100)).await;
        session.set_stop();

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_session_loot_handling() {
        let plugin = MockPlugin::new("test_session_loot_handling").with_loot();
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let options = create_test_options(Some("test_session_loot_handling".to_string()));
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        let creds = Credentials {
            target: "127.0.0.1:80".to_string(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        session.send_credentials(creds).await.unwrap();

        let worker_session = session.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables, worker_session).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        session.set_stop();

        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        // Check that loot was added to session
        let results = session.results.lock().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].get_data().get("username").unwrap(), "admin");
        assert_eq!(results[0].get_data().get("password").unwrap(), "password");
    }

    #[tokio::test]
    async fn test_concurrent_loot_addition() {
        let plugin = MockPlugin::new("test_concurrent_loot_addition").with_loot();
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let mut options = create_test_options(Some("test_concurrent_loot_addition".to_string()));
        options.concurrency = 4;
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        // Send multiple unique credentials
        for i in 0..20 {
            let creds = Credentials {
                target: format!("127.0.0.1:{}", 8000 + i),
                username: format!("user{}", i),
                password: format!("pass{}", i),
            };
            session.send_credentials(creds).await.unwrap();
        }

        // Start multiple workers
        let mut handles = vec![];
        for _ in 0..4 {
            let worker_session = session.clone();
            let unreachables = unreachables.clone();
            let handle = tokio::spawn(async move {
                worker(plugin_ref, unreachables, worker_session).await;
            });
            handles.push(handle);
        }

        tokio::time::sleep(Duration::from_millis(200)).await;
        session.set_stop();

        for handle in handles {
            tokio::time::timeout(Duration::from_secs(1), handle)
                .await
                .expect("Worker didn't finish in time")
                .expect("Worker panicked");
        }

        // All unique loots should be added
        let results = session.results.lock().unwrap();
        assert_eq!(results.len(), 20);
    }

    #[test]
    fn test_payload_strategy_display() {
        let single = PayloadStrategy::Single;
        let double = PayloadStrategy::UsernamePassword;

        assert_eq!(single.to_string(), "single");
        assert_eq!(double.to_string(), "username_and_password");
    }

    #[test]
    fn test_register_plugin_macro() {
        let mut inventory = Inventory::new();

        // Simulate what the macro does
        inventory.register("test1", MockPlugin::new("test1"));
        inventory.register("test2", MockPlugin::new("test2"));

        assert_eq!(inventory.len(), 2);
        assert!(inventory.contains_key("test1"));
        assert!(inventory.contains_key("test2"));
    }

    #[tokio::test]
    async fn test_worker_marks_target_unreachable_after_max_retries() {
        let plugin = MockPlugin::new("test_worker_marks_target_unreachable_after_max_retries")
            .with_attempt_error("Connection failed".to_string());
        let plugin_box: Box<dyn Plugin> = Box::new(plugin);
        let plugin_ref: &'static dyn Plugin = Box::leak(plugin_box);

        let mut options = create_test_options(Some(
            "test_worker_marks_target_unreachable_after_max_retries".to_string(),
        ));
        options.retries = 2;
        let session = Session::new_for_tests(options).unwrap();
        let unreachables = Arc::new(DashSet::new());

        let target = "192.168.1.1:80".to_string();
        let creds = Credentials {
            target: target.clone(),
            username: "admin".to_string(),
            password: "password".to_string(),
        };

        session.send_credentials(creds.clone()).await.unwrap();
        session.send_credentials(creds).await.unwrap(); // Send twice

        let worker_session = session.clone();
        let unreachables_clone = unreachables.clone();
        let worker_handle = tokio::spawn(async move {
            worker(plugin_ref, unreachables_clone, worker_session).await;
        });

        tokio::time::sleep(Duration::from_millis(500)).await;
        session.set_stop();

        tokio::time::timeout(Duration::from_secs(1), worker_handle)
            .await
            .expect("Worker didn't finish in time")
            .expect("Worker panicked");

        // Target should be marked as unreachable after max retries
        assert!(unreachables.contains(target.as_str()));
        // Second credential should be skipped
        assert_eq!(session.get_done(), 2);
    }
}
