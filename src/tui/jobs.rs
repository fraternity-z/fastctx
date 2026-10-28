//! Aggregated running-job state, output navigation, and bounded viewports.

use crate::shell::jobs::{JobSourceSummary, JobSummary, JobSummaryStatus, JobTail};
use ratatui::text::Line;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

/// Escapes terminal controls without allocating for ordinary output lines.
pub(crate) fn display_output_line(value: &str) -> Cow<'_, str> {
    if value.chars().all(|character| !character.is_control()) {
        return Cow::Borrowed(value);
    }
    Cow::Owned(
        value
            .chars()
            .flat_map(|character| {
                if character.is_control() {
                    character.escape_default().collect::<Vec<_>>()
                } else {
                    vec![character]
                }
            })
            .collect(),
    )
}

/// Complete data model for the current user's cross-session job dashboard.
#[derive(Clone, Debug)]
pub(crate) enum JobsState {
    Loading,
    Ready(Arc<[JobSummary]>),
    Empty,
    PermissionDenied(String),
    Error(String),
}

impl JobsState {
    pub(crate) fn ready(jobs: Vec<JobSummary>) -> Self {
        Self::Ready(jobs.into())
    }

    pub(crate) fn jobs(&self) -> &[JobSummary] {
        match self {
            Self::Ready(jobs) => jobs.as_ref(),
            Self::Loading | Self::Empty | Self::PermissionDenied(_) | Self::Error(_) => &[],
        }
    }
}

/// One source session and its running jobs, preserving registry order inside each group.
#[derive(Debug)]
pub(crate) struct JobGroup<'a> {
    pub(crate) source: &'a JobSourceSummary,
    pub(crate) jobs: Vec<&'a JobSummary>,
    pub(crate) total: usize,
}

/// Groups running jobs by immutable source identity; terminal records never enter the dashboard.
pub(crate) fn grouped_jobs(jobs: &[JobSummary]) -> Vec<JobGroup<'_>> {
    let mut groups = Vec::<JobGroup<'_>>::new();
    let mut group_indices = HashMap::<&str, usize>::new();
    for job in jobs
        .iter()
        .filter(|job| job.status == JobSummaryStatus::Running)
    {
        let group_index = match group_indices.get(job.source.key.as_str()) {
            Some(index) => *index,
            None => {
                let index = groups.len();
                group_indices.insert(job.source.key.as_str(), index);
                groups.push(JobGroup {
                    source: &job.source,
                    jobs: Vec::new(),
                    total: 0,
                });
                index
            }
        };
        let group = &mut groups[group_index];
        group.total = group.total.saturating_add(1);
        group.jobs.push(job);
    }
    groups
}

pub(crate) fn visible_jobs(jobs: &[JobSummary]) -> Vec<&JobSummary> {
    grouped_jobs(jobs)
        .into_iter()
        .flat_map(|group| group.jobs)
        .collect()
}

pub(crate) fn visible_job_count(jobs: &[JobSummary]) -> usize {
    visible_jobs(jobs).len()
}

pub(crate) fn source_count(jobs: &[JobSummary]) -> usize {
    jobs.iter()
        .map(|job| job.source.key.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

/// Read-only output detail and viewport for the focused job.
#[derive(Clone, Debug)]
pub(crate) struct JobsDetail {
    pub(crate) job_id: Option<String>,
    pub(crate) tail: JobTail,
    pub(crate) error: Option<String>,
    pub(crate) horizontal_offset: usize,
    pub(crate) lines_below: usize,
    pub(crate) follow_tail: bool,
}

impl Default for JobsDetail {
    fn default() -> Self {
        Self {
            job_id: None,
            tail: JobTail::default(),
            error: None,
            horizontal_offset: 0,
            lines_below: 0,
            follow_tail: true,
        }
    }
}

impl JobsDetail {
    pub(crate) fn move_horizontal(&mut self, forward: bool) {
        const STEP: usize = 8;
        if forward {
            let max_offset = self
                .tail
                .lines
                .iter()
                .map(|line| {
                    Line::from(display_output_line(line))
                        .width()
                        .saturating_sub(1)
                })
                .max()
                .unwrap_or(0);
            self.horizontal_offset = self.horizontal_offset.saturating_add(STEP).min(max_offset);
        } else {
            self.horizontal_offset = self.horizontal_offset.saturating_sub(STEP);
        }
    }

    pub(crate) fn page_output(&mut self, toward_tail: bool) {
        const PAGE: usize = 8;
        if toward_tail {
            self.lines_below = self.lines_below.saturating_sub(PAGE);
            if self.lines_below == 0 {
                self.follow_tail = true;
            }
        } else {
            self.follow_tail = false;
            self.lines_below = self.lines_below.saturating_add(PAGE);
        }
    }

    pub(crate) fn jump_to_output_edge(&mut self, tail: bool) {
        if tail {
            self.lines_below = 0;
            self.follow_tail = true;
        } else {
            self.follow_tail = false;
            self.lines_below = self.tail.lines.len();
        }
    }

    pub(crate) fn toggle_follow(&mut self) {
        self.follow_tail = !self.follow_tail;
        if self.follow_tail {
            self.lines_below = 0;
        }
    }

    pub(crate) fn preserve_view_after_append(&mut self, appended: usize) {
        if !self.follow_tail {
            self.lines_below = self.lines_below.saturating_add(appended);
        }
    }
}

/// Bounded dashboard viewport whose offset is anchored to rendered rows.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JobsViewport {
    offset: usize,
}

/// Content window and edge markers for one render.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct JobsViewportWindow {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) show_above: bool,
    pub(crate) show_below: bool,
}

impl JobsViewport {
    /// Keeps the focused job row visible while reserving marker rows when possible.
    pub(crate) fn window(
        &mut self,
        focused: usize,
        total_rows: usize,
        visible_rows: usize,
    ) -> JobsViewportWindow {
        if total_rows == 0 || visible_rows == 0 {
            self.offset = 0;
            return JobsViewportWindow::default();
        }
        let focused = focused.min(total_rows - 1);
        let marker_capacity = visible_rows.saturating_sub(1);
        let content_capacity = marker_capacity.max(1);

        if focused < self.offset {
            self.offset = focused;
        } else if focused >= self.offset.saturating_add(content_capacity) {
            self.offset = focused.saturating_add(1).saturating_sub(content_capacity);
        }
        self.offset = self.offset.min(total_rows.saturating_sub(content_capacity));

        let mut start = self.offset;
        let mut end = start.saturating_add(content_capacity).min(total_rows);
        let mut show_above = start > 0;
        let mut show_below = end < total_rows;

        while end.saturating_sub(start) + usize::from(show_above) + usize::from(show_below)
            > visible_rows
        {
            if focused + 1 < end {
                end -= 1;
            } else if start < focused {
                start += 1;
            } else {
                break;
            }
            show_above = start > 0;
            show_below = end < total_rows;
        }
        self.offset = start;
        JobsViewportWindow {
            start,
            end,
            show_above,
            show_below,
        }
    }
}
