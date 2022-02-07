use crate::app::{tim8_cc, PwmMonitor};
use rtic::mutex_prelude::*;

pub(crate) fn tim8_cc(mut context: tim8_cc::Context) {
    let monitor: &PwmMonitor = &context.local.monitor;

    // First, check that this interrupt is a valid capture, since this interrupt
    // fires twice per period. If not, bail out to speed up the interrupt.
    if !monitor.is_valid_capture() {
        return;
    }

    // observe duty cycle
    // This is done up here to minimize time in the critical section.
    let observation = monitor.get_duty_cycle();

    // entering critical section
    context.shared.last_observed_turret_position.lock(|guard| {
        // update the shared state
        *guard = observation;
    });
    // leaving critical section
}
