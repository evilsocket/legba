use std::sync::Arc;
use std::time;

use human_bytes::human_bytes;
use memory_stats::memory_stats;

use crate::Session;

pub(crate) fn statistics(session: Arc<Session>) {
    let one_sec = time::Duration::from_millis(1000);
    while !session.is_stop() {
        std::thread::sleep(one_sec);

        let total = session.get_total();
        let done = session.get_done();
        let perc = (done as f32 / total as f32) * 100.0;
        let errors = session.get_errors();
        let speed = session.get_speed();
        let memory = if let Some(usage) = memory_stats() {
            usage.physical_mem
        } else {
            log::error!("couldn't get the current memory usage");
            0
        };

        if errors > 0 {
            log::info!(
                "tasks={} mem={} targets={} attempts={} done={} ({:.2?}%) errors={} speed={:.2?} reqs/s",
                session.options.concurrency,
                human_bytes(memory as f64),
                session.targets.len(),
                total,
                done,
                perc,
                errors,
                speed,
            );
        } else {
            log::info!(
                "tasks={} mem={} targets={} attempts={} done={} ({:.2?}%) speed={:.2?} reqs/s",
                session.options.concurrency,
                human_bytes(memory as f64),
                session.targets.len(),
                total,
                done,
                perc,
                speed,
            );
        }
    }
}
