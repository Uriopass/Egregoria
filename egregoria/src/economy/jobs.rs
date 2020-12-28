use crate::economy::Market;
use legion::system;

pub struct JobApplication;

#[system]
pub fn job_market_update(#[resource] jobs: &mut Market<JobApplication>) {
    for trade in jobs.make_trades() {
        log::info!("job created {:?}", trade);
    }
}
