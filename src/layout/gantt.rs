use std::collections::HashMap;

use log::debug;

use crate::model::gantt::GanttDiagram;
use crate::model::richtext::plain_text;
use crate::parser::creole::parse_creole;
use crate::Result;

// ---------------------------------------------------------------------------
// Layout output types
// ---------------------------------------------------------------------------

/// Fully positioned Gantt chart ready for rendering.
#[derive(Debug)]
pub struct GanttLayout {
    pub bars: Vec<GanttBarLayout>,
    pub dependencies: Vec<GanttDepLayout>,
    pub notes: Vec<GanttNoteLayout>,
    pub time_axis: GanttTimeAxis,
    pub width: f64,
    pub height: f64,
}

/// A single positioned task bar.
#[derive(Debug)]
pub struct GanttBarLayout {
    pub id: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<String>,
}

/// A dependency arrow between two task bars.
#[derive(Debug)]
pub struct GanttDepLayout {
    pub from: String,
    pub to: String,
    pub points: Vec<(f64, f64)>,
}

/// A note positioned relative to a task bar or the overall chart.
#[derive(Debug, Clone)]
pub struct GanttNoteLayout {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub connector: Option<(f64, f64, f64, f64)>,
}

/// The time axis at the top of the chart.
#[derive(Debug)]
pub struct GanttTimeAxis {
    pub labels: Vec<GanttTimeLabel>,
    pub y: f64,
}

/// A single label on the time axis.
#[derive(Debug)]
pub struct GanttTimeLabel {
    pub text: String,
    pub x: f64,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const DAY_WIDTH: f64 = 20.0;
const BAR_HEIGHT: f64 = 20.0;
const ROW_HEIGHT: f64 = 30.0;
const LABEL_AREA_WIDTH: f64 = 160.0;
const MARGIN: f64 = 20.0;
const TIME_AXIS_HEIGHT: f64 = 30.0;
const CHAR_WIDTH: f64 = 7.2;
const NOTE_GAP: f64 = 16.0;
const NOTE_PAD_H: f64 = 8.0;
const NOTE_PAD_V: f64 = 6.0;
const MIN_NOTE_WIDTH: f64 = 60.0;
const MIN_NOTE_HEIGHT: f64 = 28.0;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the effective ID for a task (alias if set, otherwise name).
fn task_id(task: &crate::model::gantt::GanttTask) -> &str {
    task.alias.as_deref().unwrap_or(&task.name)
}

/// Compute the start day offset for each task, resolving dependencies.
///
/// Returns a map from task effective ID to start day offset.
fn compute_schedule(diagram: &GanttDiagram) -> HashMap<String, u32> {
    let mut start_days: HashMap<String, u32> = HashMap::new();

    // Initialize all tasks to start at day 0
    for task in &diagram.tasks {
        let id = task_id(task).to_string();
        start_days.insert(id, 0);
    }

    // Build a duration lookup: id -> duration
    let duration_map: HashMap<String, u32> = diagram
        .tasks
        .iter()
        .map(|t| (task_id(t).to_string(), t.duration_days))
        .collect();

    // Also map full name -> id for dependency resolution
    let name_to_id: HashMap<String, String> = diagram
        .tasks
        .iter()
        .map(|t| (t.name.clone(), task_id(t).to_string()))
        .collect();

    // Resolve dependencies: if A -> B, then B starts at A's end
    // Simple iterative resolution (handles chains)
    let max_iterations = diagram.tasks.len() + 1;
    for _ in 0..max_iterations {
        let mut changed = false;
        for dep in &diagram.dependencies {
            // Resolve from/to to effective IDs
            let from_id = name_to_id
                .get(&dep.from)
                .cloned()
                .unwrap_or_else(|| dep.from.clone());
            let to_id = name_to_id
                .get(&dep.to)
                .cloned()
                .unwrap_or_else(|| dep.to.clone());

            let from_start = *start_days.get(&from_id).unwrap_or(&0);
            let from_dur = *duration_map.get(&from_id).unwrap_or(&0);
            let from_end = from_start + from_dur;

            let current_start = start_days.get(&to_id).copied().unwrap_or(0);
            if from_end > current_start {
                start_days.insert(to_id, from_end);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    start_days
}

/// Compute the total number of days across all tasks.
fn total_days(diagram: &GanttDiagram, schedule: &HashMap<String, u32>) -> u32 {
    let mut max_end: u32 = 0;
    for task in &diagram.tasks {
        let id = task_id(task).to_string();
        let start = schedule.get(&id).copied().unwrap_or(0);
        let end = start + task.duration_days;
        if end > max_end {
            max_end = end;
        }
    }
    max_end.max(1) // at least 1 day
}

fn note_size(text: &str) -> (f64, f64) {
    let plain = plain_text(&parse_creole(text)).replace("\\n", "\n");
    let lines: Vec<&str> = plain.lines().collect();
    let max_width = lines
        .iter()
        .map(|line| line.chars().count() as f64 * CHAR_WIDTH)
        .fold(0.0_f64, f64::max);
    let width = (max_width + 2.0 * NOTE_PAD_H).max(MIN_NOTE_WIDTH);
    let height = (lines.len().max(1) as f64 * 16.0 + 2.0 * NOTE_PAD_V).max(MIN_NOTE_HEIGHT);
    (width, height)
}

fn layout_notes(
    diagram: &GanttDiagram,
    bar_positions: &HashMap<String, (f64, f64, f64, f64)>,
    name_to_id: &HashMap<String, String>,
    chart_x: f64,
    chart_y: f64,
    chart_width: f64,
    chart_height: f64,
) -> Vec<GanttNoteLayout> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut notes = Vec::new();

    for note in &diagram.notes {
        let (width, height) = note_size(&note.text);
        let target_rect = note.target.as_ref().and_then(|target| {
            let target_id = name_to_id
                .get(target)
                .cloned()
                .unwrap_or_else(|| target.clone());
            bar_positions.get(&target_id).copied()
        });
        let key = format!(
            "{}:{}",
            note.target.as_deref().unwrap_or("_"),
            note.position
        );
        let stack_index = {
            let count = counts.entry(key).or_insert(0);
            let current = *count as f64;
            *count += 1;
            current
        };

        let (x, y, connector) = if let Some((bx, by, bw, bh)) = target_rect {
            let cx = bx + bw / 2.0;
            let cy = by + bh / 2.0;
            match note.position.as_str() {
                "left" => {
                    let x = bx - NOTE_GAP - width;
                    let y = by + stack_index * (height + NOTE_GAP);
                    (x, y, Some((bx, cy, x + width, y + height / 2.0)))
                }
                "top" => {
                    let x = cx - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                    let y = by - NOTE_GAP - height;
                    (x, y, Some((cx, by, x + width / 2.0, y + height)))
                }
                "right" => {
                    let x = bx + bw + NOTE_GAP;
                    let y = by + stack_index * (height + NOTE_GAP);
                    (x, y, Some((bx + bw, cy, x, y + height / 2.0)))
                }
                _ => {
                    let x = cx - width / 2.0 + stack_index * (NOTE_GAP + 20.0);
                    let y = by + bh + NOTE_GAP;
                    (x, y, Some((cx, by + bh, x + width / 2.0, y)))
                }
            }
        } else {
            match note.position.as_str() {
                "left" => (MARGIN, chart_y + stack_index * (height + NOTE_GAP), None),
                "top" => (chart_x + stack_index * (width + NOTE_GAP), MARGIN, None),
                "right" => (
                    chart_x + chart_width + NOTE_GAP,
                    chart_y + stack_index * (height + NOTE_GAP),
                    None,
                ),
                _ => (
                    chart_x + stack_index * (width + NOTE_GAP),
                    chart_y + chart_height + NOTE_GAP,
                    None,
                ),
            }
        };

        notes.push(GanttNoteLayout {
            text: note.text.clone(),
            x,
            y,
            width,
            height,
            connector,
        });
    }

    notes
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Perform layout for a Gantt chart diagram.
pub fn layout_gantt(diagram: &GanttDiagram) -> Result<GanttLayout> {
    debug!(
        "layout_gantt: {} tasks, {} deps",
        diagram.tasks.len(),
        diagram.dependencies.len()
    );

    let schedule = compute_schedule(diagram);
    let total = total_days(diagram, &schedule);

    let scale_factor = diagram.scale.unwrap_or(1) as f64;
    let day_w = DAY_WIDTH * scale_factor;

    // Compute label area width based on longest task name
    let max_label_len = diagram
        .tasks
        .iter()
        .map(|t| t.name.len())
        .max()
        .unwrap_or(0);
    let label_area = (max_label_len as f64 * CHAR_WIDTH + 2.0 * MARGIN).max(LABEL_AREA_WIDTH);

    let chart_x = MARGIN + label_area;
    let chart_y = MARGIN + TIME_AXIS_HEIGHT;

    // --- Bars ---
    let mut bars: Vec<GanttBarLayout> = Vec::new();
    let mut bar_positions: HashMap<String, (f64, f64, f64, f64)> = HashMap::new();

    for (row, task) in diagram.tasks.iter().enumerate() {
        let id = task_id(task).to_string();
        let start_day = schedule.get(&id).copied().unwrap_or(0);

        let x = chart_x + start_day as f64 * day_w;
        let y = chart_y + row as f64 * ROW_HEIGHT;
        let w = task.duration_days as f64 * day_w;
        let h = BAR_HEIGHT;

        debug!("  bar '{id}' start_day={start_day} x={x:.1} y={y:.1} w={w:.1}");

        bar_positions.insert(id.clone(), (x, y, w, h));
        bars.push(GanttBarLayout {
            id: id.clone(),
            label: task.name.clone(),
            x,
            y,
            width: w,
            height: h,
            color: task.color.clone(),
        });
    }

    // --- Dependencies ---
    let name_to_id: HashMap<String, String> = diagram
        .tasks
        .iter()
        .map(|t| (t.name.clone(), task_id(t).to_string()))
        .collect();

    let mut dep_layouts: Vec<GanttDepLayout> = Vec::new();
    for dep in &diagram.dependencies {
        let from_id = name_to_id
            .get(&dep.from)
            .cloned()
            .unwrap_or_else(|| dep.from.clone());
        let to_id = name_to_id
            .get(&dep.to)
            .cloned()
            .unwrap_or_else(|| dep.to.clone());

        if let (Some(&(fx, fy, fw, fh)), Some(&(tx, ty, _tw, th))) =
            (bar_positions.get(&from_id), bar_positions.get(&to_id))
        {
            // Arrow from the end of the from-bar to the start of the to-bar
            let from_x = fx + fw;
            let from_y = fy + fh / 2.0;
            let to_x = tx;
            let to_y = ty + th / 2.0;

            let mid_x = from_x + (to_x - from_x) / 2.0;
            let points = vec![
                (from_x, from_y),
                (mid_x, from_y),
                (mid_x, to_y),
                (to_x, to_y),
            ];

            debug!("  dep '{from_id}' -> '{to_id}': {points:?}");

            dep_layouts.push(GanttDepLayout {
                from: from_id,
                to: to_id,
                points,
            });
        }
    }

    // --- Time axis ---
    let time_labels = build_time_axis(total, day_w, chart_x, diagram);

    let mut time_axis = GanttTimeAxis {
        labels: time_labels,
        y: MARGIN,
    };

    // --- Total dimensions ---
    let chart_width = total as f64 * day_w;
    let chart_height = diagram.tasks.len() as f64 * ROW_HEIGHT;
    let mut notes = layout_notes(
        diagram,
        &bar_positions,
        &name_to_id,
        chart_x,
        chart_y,
        chart_width,
        chart_height,
    );

    let mut min_x = MARGIN;
    let mut min_y = MARGIN;
    let mut max_x = chart_x + chart_width;
    let mut max_y = chart_y + chart_height;
    for note in &notes {
        min_x = min_x.min(note.x);
        min_y = min_y.min(note.y);
        max_x = max_x.max(note.x + note.width);
        max_y = max_y.max(note.y + note.height);
    }

    let shift_x = if min_x < MARGIN { MARGIN - min_x } else { 0.0 };
    let shift_y = if min_y < MARGIN { MARGIN - min_y } else { 0.0 };

    if shift_x > 0.0 || shift_y > 0.0 {
        for bar in &mut bars {
            bar.x += shift_x;
            bar.y += shift_y;
        }
        for dep in &mut dep_layouts {
            for (x, y) in &mut dep.points {
                *x += shift_x;
                *y += shift_y;
            }
        }
        for label in &mut time_axis.labels {
            label.x += shift_x;
        }
        time_axis.y += shift_y;
        for note in &mut notes {
            note.x += shift_x;
            note.y += shift_y;
            if let Some((x1, y1, x2, y2)) = note.connector.as_mut() {
                *x1 += shift_x;
                *x2 += shift_x;
                *y1 += shift_y;
                *y2 += shift_y;
            }
        }
        max_x += shift_x;
        max_y += shift_y;
    }

    let width = max_x + MARGIN;
    let height = max_y + MARGIN;

    debug!("layout_gantt done: {width:.0}x{height:.0}");

    Ok(GanttLayout {
        bars,
        dependencies: dep_layouts,
        notes,
        time_axis,
        width,
        height,
    })
}

/// Build time axis labels.
fn build_time_axis(
    total_days: u32,
    day_w: f64,
    chart_x: f64,
    diagram: &GanttDiagram,
) -> Vec<GanttTimeLabel> {
    let mut labels = Vec::new();

    let is_weekly = diagram
        .print_scale
        .as_deref()
        .is_some_and(|s| s.eq_ignore_ascii_case("weekly"));

    let step = if is_weekly { 7 } else { 1 };
    let max_labels = if is_weekly {
        (total_days / 7) + 2
    } else {
        total_days + 1
    };

    let mut day = 0u32;
    let mut label_count = 0u32;
    while day <= total_days && label_count < max_labels {
        let x = chart_x + day as f64 * day_w;
        let text = if is_weekly {
            format!("W{}", day / 7 + 1)
        } else {
            format!("D{}", day + 1)
        };
        labels.push(GanttTimeLabel { text, x });
        day += step;
        label_count += 1;
    }

    labels
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::gantt::{GanttDependency, GanttDiagram, GanttNote, GanttTask};

    fn empty_diagram() -> GanttDiagram {
        GanttDiagram {
            tasks: vec![],
            dependencies: vec![],
            project_start: None,
            closed_days: vec![],
            colored_ranges: vec![],
            scale: None,
            print_scale: None,
            notes: vec![],
        }
    }

    fn simple_task(name: &str, days: u32) -> GanttTask {
        GanttTask {
            name: name.to_string(),
            alias: None,
            duration_days: days,
            color: None,
            start_date: None,
        }
    }

    fn task_with_alias(name: &str, alias: &str, days: u32) -> GanttTask {
        GanttTask {
            name: name.to_string(),
            alias: Some(alias.to_string()),
            duration_days: days,
            color: None,
            start_date: None,
        }
    }

    // 1. Empty diagram
    #[test]
    fn test_empty_diagram() {
        let d = empty_diagram();
        let layout = layout_gantt(&d).unwrap();
        assert!(layout.bars.is_empty());
        assert!(layout.dependencies.is_empty());
        assert!(layout.width > 0.0);
        assert!(layout.height > 0.0);
    }

    // 2. Single task
    #[test]
    fn test_single_task() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("Design", 10));
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.bars.len(), 1);
        let bar = &layout.bars[0];
        assert_eq!(bar.id, "Design");
        assert_eq!(bar.label, "Design");
        assert_eq!(bar.width, 10.0 * DAY_WIDTH);
        assert_eq!(bar.height, BAR_HEIGHT);
    }

    // 3. Multiple tasks stack vertically
    #[test]
    fn test_multiple_tasks_vertical() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 5));
        d.tasks.push(simple_task("B", 3));
        d.tasks.push(simple_task("C", 7));
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.bars.len(), 3);

        let y0 = layout.bars[0].y;
        let y1 = layout.bars[1].y;
        let y2 = layout.bars[2].y;
        assert!(y0 < y1, "A should be above B");
        assert!(y1 < y2, "B should be above C");
        assert!((y1 - y0 - ROW_HEIGHT).abs() < 0.01);
    }

    // 4. Dependency scheduling
    #[test]
    fn test_dependency_scheduling() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 5));
        d.tasks.push(simple_task("B", 3));
        d.dependencies.push(GanttDependency {
            from: "A".to_string(),
            to: "B".to_string(),
        });
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.bars.len(), 2);

        let a = &layout.bars[0];
        let b = &layout.bars[1];
        // B should start at A's end
        let expected_b_x = a.x + a.width;
        assert!(
            (b.x - expected_b_x).abs() < 0.01,
            "B.x={} should be A.x + A.w={}",
            b.x,
            expected_b_x
        );
    }

    // 5. Chain dependencies
    #[test]
    fn test_chain_dependencies() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 3));
        d.tasks.push(simple_task("B", 4));
        d.tasks.push(simple_task("C", 2));
        d.dependencies.push(GanttDependency {
            from: "A".to_string(),
            to: "B".to_string(),
        });
        d.dependencies.push(GanttDependency {
            from: "B".to_string(),
            to: "C".to_string(),
        });
        let layout = layout_gantt(&d).unwrap();

        let a = &layout.bars[0];
        let b = &layout.bars[1];
        let c = &layout.bars[2];

        let expected_b = a.x + a.width;
        let expected_c = b.x + b.width;
        assert!((b.x - expected_b).abs() < 0.01, "B starts after A");
        assert!((c.x - expected_c).abs() < 0.01, "C starts after B");
    }

    // 6. Scale factor
    #[test]
    fn test_scale_factor() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("T", 5));
        d.scale = Some(2);
        let layout = layout_gantt(&d).unwrap();
        let bar = &layout.bars[0];
        assert_eq!(bar.width, 5.0 * DAY_WIDTH * 2.0);
    }

    // 7. Dependency layout produces points
    #[test]
    fn test_dependency_points() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 5));
        d.tasks.push(simple_task("B", 3));
        d.dependencies.push(GanttDependency {
            from: "A".to_string(),
            to: "B".to_string(),
        });
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.dependencies.len(), 1);
        let dep = &layout.dependencies[0];
        assert!(!dep.points.is_empty());
        assert_eq!(dep.from, "A");
        assert_eq!(dep.to, "B");
    }

    // 8. Time axis labels (daily)
    #[test]
    fn test_time_axis_daily() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("T", 5));
        let layout = layout_gantt(&d).unwrap();
        // Should have labels for D1 through D6 (0..=5)
        assert!(!layout.time_axis.labels.is_empty());
        assert_eq!(layout.time_axis.labels[0].text, "D1");
    }

    // 9. Time axis labels (weekly)
    #[test]
    fn test_time_axis_weekly() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("T", 21));
        d.print_scale = Some("weekly".to_string());
        let layout = layout_gantt(&d).unwrap();
        assert!(!layout.time_axis.labels.is_empty());
        assert_eq!(layout.time_axis.labels[0].text, "W1");
        // Should have W1, W2, W3, W4
        assert!(layout.time_axis.labels.len() >= 3);
    }

    // 10. Task with alias resolves properly
    #[test]
    fn test_alias_in_layout() {
        let mut d = empty_diagram();
        d.tasks
            .push(task_with_alias("Prototype design", "TASK1", 25));
        d.tasks.push(simple_task("Testing", 5));
        d.dependencies.push(GanttDependency {
            from: "TASK1".to_string(),
            to: "Testing".to_string(),
        });
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.bars.len(), 2);
        assert_eq!(layout.bars[0].id, "TASK1");
        assert_eq!(layout.bars[0].label, "Prototype design");

        // Testing should start after TASK1 ends
        let task1 = &layout.bars[0];
        let testing = &layout.bars[1];
        let expected_x = task1.x + task1.width;
        assert!(
            (testing.x - expected_x).abs() < 0.01,
            "Testing.x={} should be TASK1 end={}",
            testing.x,
            expected_x
        );
    }

    // 11. Bounding box includes all elements
    #[test]
    fn test_bounding_box() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 10));
        d.tasks.push(simple_task("B", 5));
        let layout = layout_gantt(&d).unwrap();

        for bar in &layout.bars {
            assert!(
                bar.x + bar.width <= layout.width,
                "bar right edge {} should be <= width {}",
                bar.x + bar.width,
                layout.width
            );
            assert!(
                bar.y + bar.height <= layout.height,
                "bar bottom edge {} should be <= height {}",
                bar.y + bar.height,
                layout.height
            );
        }
    }

    // 12. Task color passes through
    #[test]
    fn test_task_color() {
        let mut d = empty_diagram();
        let mut t = simple_task("Design", 10);
        t.color = Some("Lavender/LightBlue".to_string());
        d.tasks.push(t);
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.bars[0].color.as_deref(), Some("Lavender/LightBlue"));
    }

    // 13. Total days calculation
    #[test]
    fn test_total_days() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 5));
        d.tasks.push(simple_task("B", 10));
        let schedule = compute_schedule(&d);
        let total = total_days(&d, &schedule);
        assert_eq!(total, 10);
    }

    // 14. Total days with dependency chain
    #[test]
    fn test_total_days_with_deps() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("A", 5));
        d.tasks.push(simple_task("B", 10));
        d.dependencies.push(GanttDependency {
            from: "A".to_string(),
            to: "B".to_string(),
        });
        let schedule = compute_schedule(&d);
        let total = total_days(&d, &schedule);
        assert_eq!(total, 15);
    }

    // 15. Label area width adapts to long names
    #[test]
    fn test_label_area_width() {
        let mut d = empty_diagram();
        d.tasks
            .push(simple_task("Very long task name for testing purposes", 5));
        d.tasks.push(simple_task("Short", 3));
        let layout = layout_gantt(&d).unwrap();

        // The first bar's x should be offset by a label area wider than default
        let bar = &layout.bars[0];
        assert!(
            bar.x > MARGIN + LABEL_AREA_WIDTH,
            "label area should be wider than default for long names"
        );
    }

    #[test]
    fn test_note_layout_for_task() {
        let mut d = empty_diagram();
        d.tasks.push(simple_task("Design", 5));
        d.notes.push(GanttNote {
            text: "important".to_string(),
            position: "right".to_string(),
            target: Some("Design".to_string()),
        });
        let layout = layout_gantt(&d).unwrap();
        assert_eq!(layout.notes.len(), 1);
        assert!(layout.notes[0].x > layout.bars[0].x + layout.bars[0].width);
        assert!(layout.notes[0].connector.is_some());
    }
}
