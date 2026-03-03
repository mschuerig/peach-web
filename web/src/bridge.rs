use std::cell::RefCell;
use std::rc::Rc;

use domain::ports::ComparisonObserver;
use domain::training::CompletedComparison;
use domain::PerceptualProfile;

pub struct ProfileObserver(pub Rc<RefCell<PerceptualProfile>>);

impl ComparisonObserver for ProfileObserver {
    fn comparison_completed(&mut self, completed: &CompletedComparison) {
        let mut profile = self.0.borrow_mut();
        let cent_offset = completed.comparison().target_note().offset.raw_value.abs();
        profile.update(
            completed.comparison().reference_note(),
            cent_offset,
            completed.is_correct(),
        );
    }
}
