use libtest_mimic::{Arguments, Trial};

use crate::{discover::Tests, progress::{Phase, PhaseSummary, PhaseSummaryBuilder, TaskStatus}};



pub async fn run(tests: Tests)-> PhaseSummary {
    let args = Arguments::from_args();
    let mut trials = Vec::new();

    let mut bld = PhaseSummaryBuilder::new(Phase::Test);

    for test in tests.tests {
        trials.push(Trial::test(test.name.to_owned(), move || {
            (test.test_fn)();
            Ok(())
        }));
    }

    let conclusion = libtest_mimic::run(&args, trials);

    bld.add( conclusion.num_passed as usize, TaskStatus::Ok);
    bld.add( conclusion.num_ignored as usize, TaskStatus::Skipped);
    bld.add( conclusion.num_failed  as usize, TaskStatus::Failed);

    bld.build()
 }