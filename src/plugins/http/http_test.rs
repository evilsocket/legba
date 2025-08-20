// TODO: add more tests
#[cfg(test)]
mod tests {
    use reqwest::header::{CONTENT_TYPE, HeaderValue};

    use crate::{
        creds::Credentials,
        options::Options,
        plugins::{
            Plugin,
            http::{HTTP_PASSWORD_VAR, HTTP_PAYLOAD_VAR, HTTP_USERNAME_VAR},
        },
    };

    use crate::plugins::http::{HTTP, Strategy};

    #[test]
    fn test_get_target_url_adds_default_schema_and_path() {
        let mut creds = Credentials {
            target: "localhost:3000".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_adds_default_schema() {
        let mut creds = Credentials {
            target: "localhost:3000/somepath".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/somepath",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_adds_default_path() {
        let mut creds = Credentials {
            target: "https://localhost:3000".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_preserves_query() {
        let mut creds = Credentials {
            target: "localhost:3000/?foo=bar".to_owned(),
            username: String::new(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?foo=bar",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_with_username_placeholder() {
        let mut creds = Credentials {
            target: "localhost:3000/?username={USERNAME}".to_owned(),
            username: "bob".to_owned(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?username=bob",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_with_password_placeholder() {
        let mut creds = Credentials {
            target: "localhost:3000/?p={PASSWORD}".to_owned(),
            username: String::new(),
            password: "f00b4r".to_owned(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?p=f00b4r",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_with_payload_placeholder() {
        let mut creds = Credentials {
            target: "localhost:3000/?p={PAYLOAD}".to_owned(),
            username: "something".to_owned(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?p=something",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[test]
    fn test_get_target_url_interpolates_query_urlencoded() {
        let mut creds = Credentials {
            target: "localhost:3000/?p=%7BPAYLOAD%7D".to_owned(),
            username: "something".to_owned(),
            password: String::new(),
        };
        let http = HTTP::new(Strategy::Request);
        assert_eq!(
            "https://localhost:3000/?p=something",
            http.get_target_url(&mut creds).unwrap()
        );
    }

    #[tokio::test]
    async fn test_plugin_adds_default_content_type_if_post() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();

        opts.http.http_method = "POST".to_owned();
        opts.http.http_payload = Some("just a test".to_owned());

        assert_eq!(Ok(()), http.setup(&opts).await);
        assert_eq!(
            Some(&HeaderValue::from_static(
                "application/x-www-form-urlencoded"
            )),
            http.headers.get(CONTENT_TYPE)
        );
    }

    #[tokio::test]
    async fn test_plugin_preserves_user_content_type() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();

        opts.http.http_method = "POST".to_owned();
        opts.http.http_payload = Some("{\"foo\": 123}".to_owned());
        opts.http.http_headers = vec!["Content-Type=application/json".to_owned()];

        assert_eq!(Ok(()), http.setup(&opts).await);
        assert_eq!(
            Some(&HeaderValue::from_static("application/json")),
            http.headers.get(CONTENT_TYPE)
        );
    }

    #[tokio::test]
    async fn test_is_success() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success = "status == 200".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_not_success() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("nope");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success = "status == 200 && contains(body, \"login ok\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_success_match() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("sir login ok sir");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success = "status == 200 && contains(body, \"login ok\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_http_enumeration_with_cyrillic_chars() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("операция успех завершена");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success = "status == 200 && contains(body, \"успех\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let mut creds = Credentials {
            target: "localhost:3000/тест/страница".to_owned(),
            username: "пользователь".to_owned(),
            password: "пароль".to_owned(),
        };

        let target_url = http.get_target_url(&mut creds).unwrap();
        assert_eq!(
            target_url,
            "https://localhost:3000/%D1%82%D0%B5%D1%81%D1%82/%D1%81%D1%82%D1%80%D0%B0%D0%BD%D0%B8%D1%86%D0%B0"
        );

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let result = http.is_success_response(&creds, response).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_custom_code() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(666)
                .header("content-type", "text/html")
                .body("sir login ok sir");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success = "status == 666".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_not_success_custom_code() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("sir login ok sir");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success = "status == 666".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_success_with_negative_match() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("all good");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success =
            "status == 200 && !contains(body, \"wrong credentials\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_not_success_with_negative_match() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("you sent the wrong credentials, freaking moron!");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        opts.http.http_success =
            "status == 200 && !contains(body, \"wrong credentials\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_not_success_with_positive_and_negative_match() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("you sent the wrong credentials, freaking moron!");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Expression that checks for status 200, contains "credentials" but NOT "wrong credentials"
        opts.http.http_success = "status == 200 && contains(body, \"credentials\") && !contains(body, \"wrong credentials\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_success_with_positive_and_negative_match() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("i like your credentials");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Expression that checks for status 200, contains "credentials" but NOT "wrong credentials"
        opts.http.http_success = "status == 200 && contains(body, \"credentials\") && !contains(body, \"wrong credentials\")".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_with_interpolated_username() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("hello foo how are you doing?");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        // Set the expression directly with the placeholder
        http.success_expression =
            format!("status == 200 && contains(body, \"{}\")", HTTP_USERNAME_VAR);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials {
            target: String::new(),
            username: "foo".to_owned(),
            password: String::new(),
        };

        let result = http.is_success_response(&creds, response).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_with_interpolated_password() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("very cool p4ssw0rd buddy!");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        // Set the expression directly with the placeholder
        http.success_expression =
            format!("status == 200 && contains(body, \"{}\")", HTTP_PASSWORD_VAR);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials {
            target: String::new(),
            username: "foo".to_owned(),
            password: "p4ssw0rd".to_owned(),
        };

        let result = http.is_success_response(&creds, response).await;
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_with_interpolated_payload() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("totally not vulnerable <svg onload=alert(1)> to xss");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        // Set the expression directly with the placeholder
        http.success_expression =
            format!("status == 200 && contains(body, \"{}\")", HTTP_PAYLOAD_VAR);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials {
            target: String::new(),
            username: "<svg onload=alert(1)>".to_owned(),
            password: String::new(),
        };

        let result = http.is_success_response(&creds, response).await;
        assert!(result.is_some());
    }

    // Tests for check_status_codes and related methods

    #[tokio::test]
    async fn test_check_status_codes_skips_when_no_target() {
        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        // opts.target is None by default

        opts.http.http_method = "GET".to_owned();

        // This should succeed because check_status_codes skips when target is None
        assert_eq!(Ok(()), http.setup(&opts).await);

        // Verify that the HTTP client was still set up correctly
        assert_eq!(http.method, reqwest::Method::GET);
    }

    #[tokio::test]
    async fn test_check_status_codes_normal_behavior() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();

        // 200 for /, 404 for any other path
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Home page</body></html>");
        });
        let _random_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });
        let _dot_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"/\\.[a-z0-9]+$").unwrap());
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        let mut opts = Options::default();
        opts.target = Some(server.base_url());
        opts.http.http_method = "GET".to_owned();
        opts.http.http_success = "status == 200".to_owned();
        let result = http.setup(&opts).await;
        assert_eq!(Ok(()), result);
        // Don't assert mocks, just ensure no panic
    }

    #[tokio::test]
    async fn test_check_status_codes_different_strategies() {
        use httpmock::prelude::*;
        use regex::Regex;
        let strategies = vec![
            Strategy::Request,
            Strategy::Form,
            Strategy::BasicAuth,
            Strategy::Enumeration,
            Strategy::VHostEnum,
        ];
        for strategy in strategies {
            let server = MockServer::start();
            let _home_mock = server.mock(|when, then| {
                when.method(GET).path("/");
                then.status(200)
                    .header("content-type", "text/html")
                    .body("<html><body>Home page</body></html>");
            });
            let _random_mock = server.mock(|when, then| {
                when.method(GET)
                    .path_matches(Regex::new(r"/[a-z0-9]+$").unwrap());
                then.status(404)
                    .header("content-type", "text/html")
                    .body("<html><body>Not found</body></html>");
            });
            let mut http = HTTP::new(strategy);
            let mut opts = Options::default();
            opts.target = Some(server.base_url());
            opts.http.http_method = "GET".to_owned();
            opts.http.http_success = "status == 200".to_owned();
            let result = http.setup(&opts).await;
            assert_eq!(Ok(()), result);
        }
    }

    // Tests for check_false_negatives
    #[tokio::test]
    async fn test_check_false_negatives_success() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Home page</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        http.success_expression = "status == 200".to_owned();
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_negatives(&opts).await;
        assert_eq!(result, Ok(()));
        assert_eq!(http.real_target, None);
    }

    #[tokio::test]
    async fn test_check_false_negatives_404_error() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        http.success_expression = "status == 200".to_owned();
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_negatives(&opts).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.contains("success condition did not validate") && error.contains("existing page"),
            "Error message was: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_check_false_negatives_redirect_to_new_domain() {
        use httpmock::prelude::*;
        use reqwest::redirect;

        let server = MockServer::start();
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(301)
                .header("location", "https://example.com/")
                .body("");
        });

        let mut http = HTTP::new(Strategy::Request);
        // Set up client that doesn't follow redirects
        http.client = reqwest::Client::builder()
            .redirect(redirect::Policy::none())
            .build()
            .unwrap();

        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_negatives(&opts).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("adjusted to real target"));
        assert_eq!(http.real_target, Some("https://example.com".to_owned()));
    }

    #[tokio::test]
    async fn test_check_false_negatives_redirect_to_relative_path() {
        use httpmock::prelude::*;
        use reqwest::redirect;

        let server = MockServer::start();
        let _home_mock = server.mock(|when, then| {
            when.method(GET).path("/");
            then.status(302).header("location", "/login").body("");
        });

        let mut http = HTTP::new(Strategy::Request);
        // Set up client that doesn't follow redirects
        http.client = reqwest::Client::builder()
            .redirect(redirect::Policy::none())
            .build()
            .unwrap();

        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_negatives(&opts).await;
        assert_eq!(result, Ok(()));
        assert_eq!(http.real_target, None);
    }

    // Tests for check_dot_false_positives
    #[tokio::test]
    async fn test_check_dot_false_positives_success() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();
        let _dot_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"^/\.[a-z]+$").unwrap());
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_dot_false_positives(&opts, true).await;
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_check_dot_false_positives_returns_error() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();
        let _dot_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"^/\.[a-z]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Dot page content</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        http.success_expression = "status == 200".to_owned();
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_dot_false_positives(&opts, true).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("non existent page starting with a dot")
        );
    }

    #[tokio::test]
    async fn test_check_dot_false_positives_no_adjust_aborts() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();
        let _dot_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"^/\.[a-z]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Dot page content</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        http.success_expression = "status == 200".to_owned();
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_dot_false_positives(&opts, false).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("aborting due to likely false positives")
        );
    }

    // Tests for check_false_positives
    #[tokio::test]
    async fn test_check_false_positives_success() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();
        let _random_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"^/[a-z]+$").unwrap());
            then.status(404)
                .header("content-type", "text/html")
                .body("<html><body>Not found</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_positives(&opts, true).await;
        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn test_check_false_positives_returns_error() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();
        let _random_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"^/[a-z]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Random page success</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        http.success_expression = "status == 200".to_owned();
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_positives(&opts, true).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non existent page"));
    }

    #[tokio::test]
    async fn test_check_false_positives_no_adjust_aborts() {
        use httpmock::prelude::*;
        use regex::Regex;

        let server = MockServer::start();
        let _random_mock = server.mock(|when, then| {
            when.method(GET)
                .path_matches(Regex::new(r"^/[a-z]+$").unwrap());
            then.status(200)
                .header("content-type", "text/html")
                .body("<html><body>Random page success</body></html>");
        });

        let mut http = HTTP::new(Strategy::Request);
        http.success_expression = "status == 200".to_owned();
        let opts = Options {
            target: Some(server.base_url()),
            ..Options::default()
        };

        let result = http.check_false_positives(&opts, false).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("aborting due to likely false positives")
        );
    }

    #[tokio::test]
    async fn test_is_not_success_body_size_check() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        // Create a mock with a specific body size (25 bytes)
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("This is exactly 25 bytes!"); // 25 bytes including the exclamation
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // This expression should fail when body length is 25
        opts.http.http_success = "status == 200 && size != 25".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return None (failure) because body length is 25
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_success_body_size_check() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        // Create a mock with a different body size
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("This is a different length message");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // This expression should succeed when body length is NOT 25
        opts.http.http_success = "status == 200 && size != 25".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return Some (success) because body length is not 25
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_regex_matches() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("Welcome user123 to our system!");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test the str::regex_matches builtin function with a regex pattern
        opts.http.http_success =
            r#"status == 200 && str::regex_matches(body, "user[0-9]+")"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return Some (success) because body matches the regex pattern
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_not_success_regex_matches() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("Welcome guest to our system!");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test the str::regex_matches builtin function - should fail when pattern doesn't match
        opts.http.http_success =
            r#"status == 200 && str::regex_matches(body, "user[0-9]+")"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return None (failure) because body doesn't match the regex pattern
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_is_success_complex_regex_matches() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"session_id": "abc123def456", "status": "authenticated"}"#);
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test with a more complex regex pattern matching session IDs
        opts.http.http_success =
            r#"status == 200 && str::regex_matches(body, "\"session_id\":\\s*\"[a-z0-9]{12}\"")"#
                .to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return Some (success) because body contains a valid session_id pattern
        assert!(result.is_some());
    }

    // Header-based validation tests
    #[tokio::test]
    async fn test_is_success_header_check() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("x-auth-token", "secret123")
                .header("content-type", "application/json")
                .body(r#"{"status": "ok"}"#);
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test checking for specific header value
        opts.http.http_success = r#"status == 200 && x_auth_token == "secret123""#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_combined_header_body() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"result": "success"}"#);
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test combining header and body checks
        opts.http.http_success =
            r#"status == 200 && content_type == "application/json" && contains(body, "success")"#
                .to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    // Size variable tests
    #[tokio::test]
    async fn test_is_success_size_comparisons() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html")
                .body("This is a test response with some content");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test size > comparison
        opts.http.http_success = "status == 200 && size > 10".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_empty_body() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(204).body("");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test for empty body
        opts.http.http_success = "status == 204 && size == 0".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    // Complex logical expressions
    #[tokio::test]
    async fn test_is_success_complex_logical_or() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(201)
                .header("content-type", "application/json")
                .body(r#"{"message": "created ok"}"#);
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test complex OR conditions
        opts.http.http_success = "(status == 200 || status == 201) && (contains(body, \"success\") || contains(body, \"ok\"))".to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_multiple_regex_patterns() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200).body("token=abc123def expires=1234567890");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test multiple regex patterns
        opts.http.http_success = r#"status == 200 && str::regex_matches(body, "token=[a-z0-9]+") && str::regex_matches(body, "expires=[0-9]+")"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    // Special characters and edge cases
    #[tokio::test]
    async fn test_is_success_special_characters() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("content-type", "text/html; charset=utf-8")
                .body("Response with special chars: 🚀 émojis € and symbols ©");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test with special characters
        opts.http.http_success =
            r#"status == 200 && contains(body, "🚀") && contains(body, "€")"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    // Case sensitivity tests
    #[tokio::test]
    async fn test_is_success_case_insensitive_regex() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200).body("Login SUCCESSFUL");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test case-insensitive regex
        opts.http.http_success =
            r#"status == 200 && str::regex_matches(body, "(?i)success")"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    // Cookie/session validation tests
    #[tokio::test]
    async fn test_is_success_cookie_check() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("set-cookie", "session_id=abc123; Path=/; HttpOnly")
                .body("Logged in");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test for cookie presence
        opts.http.http_success =
            r#"status == 200 && contains(set_cookie, "session_id")"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_success_302_with_set_cookie() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/login");
            then.status(302)
                .header("location", "/dashboard")
                .header("set-cookie", "auth_token=xyz789; Path=/; HttpOnly")
                .body("");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test for 302 status with non-empty set_cookie
        opts.http.http_success = r#"status == 302 && set_cookie != """#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/login", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return Some (success) because status is 302 and set_cookie is not empty
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn test_is_not_success_302_without_set_cookie() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/redirect");
            then.status(302).header("location", "/somewhere").body("");
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test for 302 status with non-empty set_cookie
        opts.http.http_success = r#"status == 302 && set_cookie != """#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/redirect", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        // Should return None (failure) because set_cookie is empty (header not present)
        assert!(result.is_none());
    }

    // Interpolation with special characters
    #[tokio::test]
    async fn test_is_success_interpolation_special_chars() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .body(r#"Welcome user "test@example.com" to the system"#);
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        // Set the expression directly with the placeholder
        http.success_expression =
            format!("status == 200 && contains(body, \"{}\")", HTTP_USERNAME_VAR);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials {
            target: String::new(),
            username: "test@example.com".to_owned(),
            password: String::new(),
        };

        let result = http.is_success_response(&creds, response).await;
        assert!(result.is_some());
    }

    // Complex nested logical expressions
    #[tokio::test]
    async fn test_is_success_nested_logic() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let _mock = server.mock(|when, then| {
            when.method(GET).path("/test");
            then.status(200)
                .header("x-api-version", "2.0")
                .body(r#"{"status": "ok", "data": {"user": "admin"}}"#);
        });

        let mut http = HTTP::new(Strategy::Enumeration);
        let mut opts = Options::default();

        // Test nested logical conditions
        opts.http.http_success = r#"(status == 200 && x_api_version == "2.0") && (contains(body, "ok") && contains(body, "admin"))"#.to_owned();
        opts.http.http_method = "GET".to_owned();

        assert_eq!(Ok(()), http.setup(&opts).await);

        let response = http
            .client
            .get(format!("{}/test", server.base_url()))
            .send()
            .await
            .unwrap();

        let creds = Credentials::default();
        let result = http.is_success_response(&creds, response).await;

        assert!(result.is_some());
    }
}
