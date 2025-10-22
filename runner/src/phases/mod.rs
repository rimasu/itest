use crate::{discover::{SetUps, Tests}, progress::{ OverallResult, OverallSummaryBuilder, Phase, PhaseResult, PhaseSummary, ProgressListener, ProgressMonitor},  GlobalContext, TearDown, TearDowns};


mod set_up_phase;
mod test_phase;
mod tear_down_phase;

pub async fn run(
    ctx: &mut GlobalContext,
    set_ups: SetUps,
    tests: Tests,
    progress: &ProgressListener,
) -> OverallResult {
    let mut summary = OverallSummaryBuilder::new();

    let (tear_downs, result) = run_set_ups(ctx, set_ups, progress, &mut summary).await;

    let test_outcome = if result == PhaseResult::Ok {
        test_phase::run(tests).await
    } else {
        PhaseSummary::skipped(Phase::Test)
    };
    summary.add_phase(test_outcome);

    run_tear_downs(tear_downs, progress, &mut summary).await;

    let summary = summary.build();
    let result = summary.result;
    progress.finished(summary).await;

    result
}

async fn run_set_ups(
    ctx: &mut GlobalContext,
    set_ups: SetUps,
    progress: &ProgressListener,
    overall_summary: &mut OverallSummaryBuilder
)-> (TearDowns, PhaseResult) {

    let (tear_downs, summary) = set_up_phase::run(set_ups, ctx, progress).await;

    let result = summary.result;

    overall_summary.add_phase(summary.clone());

    (tear_downs, result)
}

async fn run_tear_downs(
    tear_downs: TearDowns,
    progress: &ProgressListener,
    overall_summary: &mut OverallSummaryBuilder
) {
    let summary = tear_down_phase::run(progress, tear_downs).await;
    
    overall_summary.add_phase(summary);
}