// layout::sequence_teoz::builder - TileBuilder + PlayingSpace orchestration
//
// Port of Java PlantUML's TileBuilder, PlayingSpace, and
// SequenceDiagramFileMakerTeoz into a single build_teoz_layout() function.
//
// Pipeline:
//   1. Create RealLine (constraint arena)
//   2. Create LivingSpaces for each participant (with Real positions)
//   3. Build Tiles from events (TileBuilder logic)
//   4. Add constraints from tiles
//   5. Compile constraints (solve)
//   6. Assign Y positions (fillPositionelTiles)
//   7. Extract SeqLayout from positioned tiles

use std::collections::HashMap;

use crate::font_metrics;
use crate::model::sequence::{
	FragmentKind, ParticipantKind, SeqArrowHead, SeqArrowStyle,
	SeqDirection, SeqEvent, SequenceDiagram,
};
use crate::skin::rose::{self, TextMetrics};
use crate::style::SkinParams;
use crate::Result;

use crate::layout::sequence::{
	ActivationLayout, DelayLayout, DestroyLayout, DividerLayout, FragmentLayout,
	MessageLayout, NoteLayout, ParticipantLayout, RefLayout, SeqLayout,
};

use super::living::LivingSpace;
use super::real::{RealId, RealLine};

// ── Constants ────────────────────────────────────────────────────────────────

const FONT_SIZE: f64 = 14.0;
const MSG_FONT_SIZE: f64 = 13.0;
const NOTE_FONT_SIZE: f64 = 13.0;
const ACTIVATION_WIDTH: f64 = 10.0;
const SELF_MSG_WIDTH: f64 = 42.0;
const NOTE_PADDING: f64 = rose::NOTE_PADDING;
const NOTE_FOLD: f64 = rose::SEQ_NOTE_FOLD;
/// Java teoz: participant heads render at y=10 (5px frame + 5px inner margin).
/// PlayingSpace content starts below the preferred participant height.
const STARTING_Y: f64 = 10.0;
/// Minimum gap between adjacent participant right-edge and next left-edge.
const PARTICIPANT_GAP: f64 = 5.0;
/// Document margin: 5px (exporter) + 5px (teoz UTranslate) = 10px.
/// Applied to all x positions, matching Java's coordinate chain.
const DOC_MARGIN_X: f64 = 10.0;

// ── Tile types (inline, simplified) ──────────────────────────────────────────

/// Simplified tile kind for the builder pipeline.
/// Each variant carries the data needed for constraint generation and
/// layout extraction. This will later be replaced by the full tile module.
#[derive(Debug)]
#[allow(dead_code)]
enum TeozTile {
	/// Normal message between two different participants
	Communication {
		from_name: String,
		to_name: String,
		from_idx: usize,
		to_idx: usize,
		text: String,
		text_lines: Vec<String>,
		is_dashed: bool,
		has_open_head: bool,
		/// Minimum pixel width needed by the message text
		text_width: f64,
		/// Preferred height of this tile
		height: f64,
		/// Y position (assigned in step 6)
		y: Option<f64>,
		/// Autonumber label if any
		autonumber: Option<String>,
		/// RealId of the source participant center
		from_center: RealId,
		/// RealId of the target participant center
		to_center: RealId,
		/// Circle decoration on from end
		circle_from: bool,
		/// Circle decoration on to end
		circle_to: bool,
	},
	/// Self-message (from == to)
	SelfMessage {
		participant_idx: usize,
		text: String,
		text_lines: Vec<String>,
		is_dashed: bool,
		has_open_head: bool,
		text_width: f64,
		height: f64,
		y: Option<f64>,
		autonumber: Option<String>,
		center: RealId,
		direction: SeqDirection,
		/// Activation level at the time of this self-message
		active_level: usize,
		/// Circle decoration on from end
		circle_from: bool,
		/// Circle decoration on to end
		circle_to: bool,
	},
	/// Activate / Deactivate / Destroy life event
	LifeEvent {
		height: f64,
		y: Option<f64>,
	},
	/// Note on a participant
	Note {
		participant_idx: usize,
		text: String,
		is_left: bool,
		width: f64,
		height: f64,
		y: Option<f64>,
		center: RealId,
	},
	/// Note spanning two participants
	NoteOver {
		participants: Vec<String>,
		text: String,
		width: f64,
		height: f64,
		y: Option<f64>,
	},
	/// Divider line
	Divider {
		text: Option<String>,
		height: f64,
		y: Option<f64>,
	},
	/// Delay section
	Delay {
		text: Option<String>,
		height: f64,
		y: Option<f64>,
	},
	/// Reference over participants
	Ref {
		participants: Vec<String>,
		label: String,
		height: f64,
		y: Option<f64>,
	},
	/// Fragment (alt/loop/opt/etc.) start
	FragmentStart {
		kind: FragmentKind,
		label: String,
		height: f64,
		y: Option<f64>,
	},
	/// Fragment separator (else)
	FragmentSeparator {
		label: String,
		height: f64,
		y: Option<f64>,
	},
	/// Fragment end
	FragmentEnd {
		height: f64,
		y: Option<f64>,
	},
	/// Spacing
	Spacing {
		pixels: f64,
		y: Option<f64>,
	},
	/// Group start (legacy)
	GroupStart {
		_label: Option<String>,
		height: f64,
		y: Option<f64>,
	},
	/// Group end (legacy)
	GroupEnd {
		height: f64,
		y: Option<f64>,
	},
}

impl TeozTile {
	fn preferred_height(&self) -> f64 {
		match self {
			Self::Communication { height, .. } => *height,
			Self::SelfMessage { height, .. } => *height,
			Self::LifeEvent { height, .. } => *height,
			Self::Note { height, .. } => *height,
			Self::NoteOver { height, .. } => *height,
			Self::Divider { height, .. } => *height,
			Self::Delay { height, .. } => *height,
			Self::Ref { height, .. } => *height,
			Self::FragmentStart { height, .. } => *height,
			Self::FragmentSeparator { height, .. } => *height,
			Self::FragmentEnd { height, .. } => *height,
			Self::Spacing { pixels, .. } => *pixels,
			Self::GroupStart { height, .. } => *height,
			Self::GroupEnd { height, .. } => *height,
		}
	}

	fn set_y(&mut self, val: f64) {
		match self {
			Self::Communication { y, .. } => *y = Some(val),
			Self::SelfMessage { y, .. } => *y = Some(val),
			Self::LifeEvent { y, .. } => *y = Some(val),
			Self::Note { y, .. } => *y = Some(val),
			Self::NoteOver { y, .. } => *y = Some(val),
			Self::Divider { y, .. } => *y = Some(val),
			Self::Delay { y, .. } => *y = Some(val),
			Self::Ref { y, .. } => *y = Some(val),
			Self::FragmentStart { y, .. } => *y = Some(val),
			Self::FragmentSeparator { y, .. } => *y = Some(val),
			Self::FragmentEnd { y, .. } => *y = Some(val),
			Self::Spacing { y, .. } => *y = Some(val),
			Self::GroupStart { y, .. } => *y = Some(val),
			Self::GroupEnd { y, .. } => *y = Some(val),
		}
	}

	fn get_y(&self) -> Option<f64> {
		match self {
			Self::Communication { y, .. } => *y,
			Self::SelfMessage { y, .. } => *y,
			Self::LifeEvent { y, .. } => *y,
			Self::Note { y, .. } => *y,
			Self::NoteOver { y, .. } => *y,
			Self::Divider { y, .. } => *y,
			Self::Delay { y, .. } => *y,
			Self::Ref { y, .. } => *y,
			Self::FragmentStart { y, .. } => *y,
			Self::FragmentSeparator { y, .. } => *y,
			Self::FragmentEnd { y, .. } => *y,
			Self::Spacing { y, .. } => *y,
			Self::GroupStart { y, .. } => *y,
			Self::GroupEnd { y, .. } => *y,
		}
	}
}

// ── Layout parameters ────────────────────────────────────────────────────────

#[allow(dead_code)]
struct TeozParams {
	message_spacing: f64,
	self_msg_height: f64,
	participant_height: f64,
	msg_line_height: f64,
	frag_header_height: f64,
	divider_height: f64,
	delay_height: f64,
	ref_height: f64,
}

impl TeozParams {
	fn compute(font_family: &str, msg_font_size: f64, part_font_size: f64) -> Self {
		let h13 = font_metrics::line_height(font_family, msg_font_size, false, false);
		let h14 = font_metrics::line_height(font_family, part_font_size, false, false);

		let arrow_tm = TextMetrics::new(7.0, 7.0, 1.0, 0.0, h13);
		let message_spacing = rose::arrow_preferred_size(&arrow_tm, 0.0, 0.0).height;

		let self_msg_height = rose::SELF_ARROW_ONLY_HEIGHT;

		// Java: ComponentRoseParticipant(style, stereo, NONE, 7, 7, 7, skinParam, display, false)
		// marginX1=7, marginX2=7, marginY=7
		// preferred_height = getTextHeight() + 1 = (lineHeight + 2*7) + 1 = 31.2969
		// But the DRAWN rect height = getTextHeight() = 30.2969 (no +1).
		// We use text_height (30.2969) as box_height for rendering consistency with puma.
		let part_tm = TextMetrics::new(7.0, 7.0, 7.0, 0.0, h14);
		let participant_preferred_h =
			rose::participant_preferred_size(&part_tm, 0.0, false, 0.0, 0.0).height;
		let participant_height = participant_preferred_h - 1.0; // text_height only (drawn rect)

		let frag_header_height = h13 + 2.0;

		let divider_tm = TextMetrics::new(0.0, 0.0, 5.0, 0.0, 0.0);
		let divider_height = rose::divider_preferred_size(&divider_tm).height;

		let delay_tm = TextMetrics::new(0.0, 0.0, 5.0, 0.0, 0.0);
		let delay_height = rose::delay_text_preferred_size(&delay_tm).height;

		let ref_height = h13 + h14 + rose::REF_HEIGHT_FOOTER + 2.0 + 0.671875;

		Self {
			message_spacing,
			self_msg_height,
			participant_height,
			msg_line_height: h13,
			frag_header_height,
			divider_height,
			delay_height,
			ref_height,
		}
	}
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn active_left_shift(level: usize) -> f64 {
	if level == 0 { 0.0 } else { ACTIVATION_WIDTH / 2.0 }
}

fn active_right_shift(level: usize) -> f64 {
	level as f64 * (ACTIVATION_WIDTH / 2.0)
}

fn live_thickness_width(level: usize) -> f64 {
	active_left_shift(level) + active_right_shift(level)
}

fn estimate_note_height(text: &str) -> f64 {
	let lines = text.lines().count().max(1) as f64;
	let lh = font_metrics::line_height("SansSerif", NOTE_FONT_SIZE, false, false);
	let h = lines * lh + 10.0; // marginY1(5) + marginY2(5)
	h.trunc().max(25.0)
}

fn estimate_note_width(text: &str) -> f64 {
	let max_line_w = text
		.lines()
		.map(|line| font_metrics::text_width(line, "SansSerif", NOTE_FONT_SIZE, false, false))
		.fold(0.0_f64, f64::max);
	let w = max_line_w + NOTE_PADDING + NOTE_PADDING / 2.0 + NOTE_FOLD + 2.0;
	w.max(30.0)
}

#[allow(dead_code)]
fn message_text_width(text: &str, font_family: &str, font_size: f64) -> f64 {
	text.split("\\n").flat_map(|s| s.split(crate::NEWLINE_CHAR))
		.map(|line| font_metrics::text_width(line, font_family, font_size, false, false))
		.fold(0.0_f64, f64::max)
}

// ── Main build function ──────────────────────────────────────────────────────

/// Build the complete Teoz layout from a parsed sequence diagram.
///
/// This is the main orchestrator matching Java's
/// SequenceDiagramFileMakerTeoz + PlayingSpace + TileBuilder.
pub fn build_teoz_layout(
	sd: &SequenceDiagram,
	skin: &SkinParams,
) -> Result<SeqLayout> {
	log::debug!(
		"build_teoz_layout: {} participants, {} events",
		sd.participants.len(),
		sd.events.len(),
	);

	// ── Resolve font/skin params ─────────────────────────────────────────
	let default_font = skin
		.get("defaultfontname")
		.map(|s| s.as_ref())
		.unwrap_or("SansSerif");
	let default_font_size: Option<f64> = skin
		.get("defaultfontsize")
		.and_then(|s| s.parse::<f64>().ok());
	let msg_font_size: f64 = default_font_size.unwrap_or(MSG_FONT_SIZE);
	let participant_font_size: f64 = skin
		.get("participantfontsize")
		.and_then(|s| s.parse::<f64>().ok())
		.or(default_font_size)
		.unwrap_or(FONT_SIZE);
	let max_message_size: Option<f64> = skin
		.get("maxmessagesize")
		.and_then(|s| s.parse::<f64>().ok());

	let tp = TeozParams::compute(default_font, msg_font_size, participant_font_size);

	// ── Step 1: Create RealLine ──────────────────────────────────────────
	let mut rl = RealLine::new();
	let xorigin = rl.create_origin();

	// ── Step 2: Create LivingSpaces ──────────────────────────────────────
	// For each participant, compute box width/height and create Real
	// constraint variables for posB (left), posC (center), posD (right).
	let n_parts = sd.participants.len();
	let mut livings: Vec<LivingSpace> = Vec::with_capacity(n_parts);
	let mut part_layouts: Vec<ParticipantLayout> = Vec::with_capacity(n_parts);
	let mut box_widths: Vec<f64> = Vec::with_capacity(n_parts);
	let mut box_heights: Vec<f64> = Vec::with_capacity(n_parts);
	let mut name_to_idx: HashMap<String, usize> = HashMap::new();

	let mut xcurrent = rl.add_at_least(xorigin, 0.0);

	for (i, p) in sd.participants.iter().enumerate() {
		let display = p.display_name.as_deref().unwrap_or(&p.name);
		let display_lines: Vec<&str> = display.split("\\n").flat_map(|s| s.split(crate::NEWLINE_CHAR)).collect();
		let num_lines = display_lines.len();
		let max_line_w = display_lines
			.iter()
			.map(|line| {
				font_metrics::text_width(line, default_font, participant_font_size, false, false)
			})
			.fold(0.0_f64, f64::max);
		let bw = rose::participant_preferred_width(&p.kind, max_line_w, 1.5);
		let participant_line_height =
			font_metrics::line_height(default_font, participant_font_size, false, false);
		let multiline_extra = if num_lines > 1 {
			participant_line_height * (num_lines - 1) as f64
		} else {
			0.0
		};
		let base_participant_height = tp.participant_height;
		let bh = match p.kind {
			ParticipantKind::Actor => base_participant_height + 45.0 + multiline_extra,
			ParticipantKind::Boundary
			| ParticipantKind::Control
			| ParticipantKind::Entity
			| ParticipantKind::Database
			| ParticipantKind::Collections
			| ParticipantKind::Queue => base_participant_height + 20.0 + multiline_extra,
			ParticipantKind::Default => base_participant_height + multiline_extra,
		};

		// Create Real variables: posB = xcurrent, posC = posB + w/2, posD = posB + w
		let pos_b = xcurrent;
		let half_w = bw / 2.0;
		let pos_c = rl.add_fixed(pos_b, half_w);
		let pos_d = rl.add_fixed(pos_b, bw);

		livings.push(LivingSpace::new(p.name.clone(), pos_b, pos_c, pos_d));
		box_widths.push(bw);
		box_heights.push(bh);
		name_to_idx.insert(p.name.clone(), i);

		// Next participant starts after posD.
		// Java: xcurrent = livingSpace.getPosD(stringBounder).addAtLeast(0);
		xcurrent = rl.add_at_least(pos_d, 0.0);
	}

	// ── Step 3: Build tiles from events ──────────────────────────────────
	let mut tiles: Vec<TeozTile> = Vec::new();
	let mut autonumber_enabled = false;
	let mut autonumber_counter: u32 = 1;
	let mut autonumber_start: u32 = 1;
	let mut active_levels: HashMap<String, usize> = HashMap::new();

	for event in &sd.events {
		match event {
			SeqEvent::AutoNumber { start } => {
				autonumber_enabled = true;
				if let Some(n) = start {
					autonumber_counter = *n;
					autonumber_start = *n;
				}
			}
			SeqEvent::Message(msg) => {
				let autonumber = if autonumber_enabled {
					let label = format!("{autonumber_counter}");
					autonumber_counter += 1;
					Some(label)
				} else {
					None
				};

				let autonumber_extra_w = autonumber.as_ref().map_or(0.0, |num| {
					font_metrics::text_width(num, default_font, msg_font_size, true, false)
						+ 4.0
				});

				let mut text_lines: Vec<String> =
					msg.text.split("\\n").flat_map(|s| s.split(crate::NEWLINE_CHAR)).map(ToString::to_string).collect();
				if let Some(max_w) = max_message_size {
					text_lines = text_lines
						.into_iter()
						.flat_map(|line| wrap_text_to_width(&line, max_w, default_font, msg_font_size))
						.collect();
				}
				let text_w = text_lines
					.iter()
					.map(|line| font_metrics::text_width(line, default_font, msg_font_size, false, false))
					.fold(0.0_f64, f64::max)
					+ autonumber_extra_w;

				let text_h = tp.msg_line_height * text_lines.len().max(1) as f64;
				let is_dashed = msg.arrow_style == SeqArrowStyle::Dashed;
				let has_open_head = matches!(msg.arrow_head, SeqArrowHead::Open | SeqArrowHead::HalfTop | SeqArrowHead::HalfBottom);

				if msg.from == msg.to {
					// Self-message
					let idx = name_to_idx.get(&msg.from).copied().unwrap_or(0);
					let center = livings[idx].pos_c;
					let tm = TextMetrics::new(7.0, 7.0, 1.0, text_w, text_h);
					let height = rose::self_arrow_preferred_size(&tm).height;

					let level = active_levels.get(&msg.from).copied().unwrap_or(0);
					tiles.push(TeozTile::SelfMessage {
						participant_idx: idx,
						text: msg.text.clone(),
						text_lines,
						is_dashed,
						has_open_head,
						text_width: text_w,
						height,
						y: None,
						autonumber,
						center,
						direction: msg.direction.clone(),
						active_level: level,
						circle_from: msg.circle_from,
						circle_to: msg.circle_to,
					});
				} else {
					// Normal message
					let fi = name_to_idx.get(&msg.from).copied().unwrap_or(0);
					let ti = name_to_idx.get(&msg.to).copied().unwrap_or(0);
					let from_center = livings[fi].pos_c;
					let to_center = livings[ti].pos_c;

					let tm = TextMetrics::new(7.0, 7.0, 1.0, text_w, text_h);
					let height = rose::arrow_preferred_size(&tm, 0.0, 0.0).height;

					tiles.push(TeozTile::Communication {
						from_name: msg.from.clone(),
						to_name: msg.to.clone(),
						from_idx: fi,
						to_idx: ti,
						text: msg.text.clone(),
						text_lines,
						is_dashed,
						has_open_head,
						text_width: text_w,
						height,
						y: None,
						autonumber,
						from_center,
						to_center,
						circle_from: msg.circle_from,
						circle_to: msg.circle_to,
					});
				}
			}
			SeqEvent::Activate(name) => {
				let level = active_levels.entry(name.clone()).or_insert(0);
				*level += 1;
				tiles.push(TeozTile::LifeEvent {
					height: 0.0,
					y: None,
				});
			}
			SeqEvent::Deactivate(name) => {
				let level = active_levels.entry(name.clone()).or_insert(0);
				if *level > 0 {
					*level -= 1;
				}
				tiles.push(TeozTile::LifeEvent {
					height: 0.0,
					y: None,
				});
			}
			SeqEvent::Destroy(_name) => {
				tiles.push(TeozTile::LifeEvent {
					height: 0.0,
					y: None,
				});
			}
			SeqEvent::NoteRight { participant, text } => {
				let idx = name_to_idx.get(participant).copied().unwrap_or(0);
				let center = livings[idx].pos_c;
				let w = estimate_note_width(text);
				let h = estimate_note_height(text);
				tiles.push(TeozTile::Note {
					participant_idx: idx,
					text: text.clone(),
					is_left: false,
					width: w,
					height: h,
					y: None,
					center,
				});
			}
			SeqEvent::NoteLeft { participant, text } => {
				let idx = name_to_idx.get(participant).copied().unwrap_or(0);
				let center = livings[idx].pos_c;
				let w = estimate_note_width(text);
				let h = estimate_note_height(text);
				tiles.push(TeozTile::Note {
					participant_idx: idx,
					text: text.clone(),
					is_left: true,
					width: w,
					height: h,
					y: None,
					center,
				});
			}
			SeqEvent::NoteOver { participants, text } => {
				let w = estimate_note_width(text);
				let h = estimate_note_height(text);
				tiles.push(TeozTile::NoteOver {
					participants: participants.clone(),
					text: text.clone(),
					width: w,
					height: h,
					y: None,
				});
			}
			SeqEvent::Divider { text } => {
				tiles.push(TeozTile::Divider {
					text: text.clone(),
					height: tp.divider_height,
					y: None,
				});
			}
			SeqEvent::Delay { text } => {
				tiles.push(TeozTile::Delay {
					text: text.clone(),
					height: tp.delay_height,
					y: None,
				});
			}
			SeqEvent::Ref { participants, label } => {
				tiles.push(TeozTile::Ref {
					participants: participants.clone(),
					label: label.clone(),
					height: tp.ref_height,
					y: None,
				});
			}
			SeqEvent::FragmentStart { kind, label } => {
				tiles.push(TeozTile::FragmentStart {
					kind: kind.clone(),
					label: label.clone(),
					height: tp.frag_header_height,
					y: None,
				});
			}
			SeqEvent::FragmentSeparator { label } => {
				tiles.push(TeozTile::FragmentSeparator {
					label: label.clone(),
					height: tp.frag_header_height,
					y: None,
				});
			}
			SeqEvent::FragmentEnd => {
				tiles.push(TeozTile::FragmentEnd {
					height: 4.0,
					y: None,
				});
			}
			SeqEvent::Spacing { pixels } => {
				tiles.push(TeozTile::Spacing {
					pixels: *pixels as f64,
					y: None,
				});
			}
			SeqEvent::GroupStart { label } => {
				tiles.push(TeozTile::GroupStart {
					_label: label.clone(),
					height: tp.frag_header_height,
					y: None,
				});
			}
			SeqEvent::GroupEnd => {
				tiles.push(TeozTile::GroupEnd {
					height: 4.0,
					y: None,
				});
			}
		}
	}

	// ── Step 4: Add constraints from tiles ───────────────────────────────
	// Communication tiles constrain participant spacing.
	// Java: CommunicationTile.addConstraints() does
	//   target_center >= source_center + arrow_preferred_width
	for tile in &tiles {
		match tile {
			TeozTile::Communication {
				from_idx,
				to_idx,
				text_width,
				from_center,
				to_center,
				..
			} => {
				let fi = *from_idx;
				let ti = *to_idx;
				let arrow_tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, tp.msg_line_height);
				let arrow_w = rose::arrow_preferred_size(&arrow_tm, 0.0, 0.0).width;

				let fi_level = active_levels.get(&livings[fi].name).copied().unwrap_or(0);
				let ti_level = active_levels.get(&livings[ti].name).copied().unwrap_or(0);
				let extra = live_thickness_width(fi_level) + live_thickness_width(ti_level);
				let needed = arrow_w + extra;

				if fi < ti {
					// Left-to-right: to_center >= from_center + needed
					rl.ensure_bigger_than_with_margin(*to_center, *from_center, needed);
				} else {
					// Right-to-left: from_center >= to_center + needed
					rl.ensure_bigger_than_with_margin(*from_center, *to_center, needed);
				}
			}
			TeozTile::SelfMessage {
				participant_idx,
				text_width,
				center,
				direction,
				active_level,
				..
			} => {
				let idx = *participant_idx;
				let tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, tp.msg_line_height);
				let needed = rose::self_arrow_preferred_size(&tm).width
					+ live_thickness_width(*active_level);

				// Self messages need space in one direction from center.
				// Constrain the adjacent participant (or origin) to be far enough.
				match direction {
					SeqDirection::LeftToRight => {
						// Need space to the right
						if idx + 1 < n_parts {
							let next_center = livings[idx + 1].pos_c;
							rl.ensure_bigger_than_with_margin(next_center, *center, needed);
						}
					}
					SeqDirection::RightToLeft => {
						// Need space to the left: center >= ref + needed
						if idx > 0 {
							let prev_center = livings[idx - 1].pos_c;
							rl.ensure_bigger_than_with_margin(*center, prev_center, needed);
						} else {
							// Leftmost participant: ensure enough room from origin
							rl.ensure_bigger_than_with_margin(*center, xorigin, needed);
						}
					}
				}
			}
			TeozTile::Note {
				participant_idx,
				is_left,
				width,
				center,
				..
			} => {
				let idx = *participant_idx;
				let note_half = *width / 2.0 + 5.0;
				if *is_left {
					// Note to the left: need space before this participant
					if idx > 0 {
						let prev_center = livings[idx - 1].pos_c;
						rl.ensure_bigger_than_with_margin(*center, prev_center, note_half);
					}
				} else {
					// Note to the right: need space after this participant
					if idx + 1 < n_parts {
						let next_center = livings[idx + 1].pos_c;
						rl.ensure_bigger_than_with_margin(next_center, *center, note_half);
					}
				}
			}
			_ => {}
		}
	}

	// ── Step 5: Compile constraints ──────────────────────────────────────
	rl.compile();

	// ── Step 6: Assign Y positions (fillPositionelTiles) ─────────────────
	// Simple linear walk: y starts at the participant box bottom + starting_y.
	let max_box_height = box_heights.iter().copied().fold(0.0_f64, f64::max);
	// Java layout uses preferred height (= drawn + 1) for lifeline start
	let max_preferred_height = max_box_height + 1.0;
	let mut y = STARTING_Y + max_preferred_height;
	for tile in tiles.iter_mut() {
		tile.set_y(y);
		y += tile.preferred_height();
	}
	let lifeline_bottom = y;

	// ── Step 7: Extract SeqLayout ────────────────────────────────────────
	// Java applies UTranslate(5,5) internally + 5px exporter margin = 10px total.
	// Compute x_offset = DOC_MARGIN_X - min1 (min of origin and tile minXes).
	// For simple cases min1 = 0, so x_offset = 10.
	let origin_val = rl.get_value(xorigin);
	let min1 = livings.iter()
		.map(|l| rl.get_value(l.pos_c))
		.fold(origin_val, f64::min);
	let x_offset = DOC_MARGIN_X - min1;
	// Helper: get Real x value with document margin applied.
	let get_x = |id: RealId| -> f64 { rl.get_value(id) + x_offset };

	// Build ParticipantLayout from Real-resolved positions
	for (i, p) in sd.participants.iter().enumerate() {
		let center_x = get_x(livings[i].pos_c);
		part_layouts.push(ParticipantLayout {
			name: p.name.clone(),
			x: center_x,
			box_width: box_widths[i],
			box_height: box_heights[i],
			kind: p.kind.clone(),
			color: p.color.clone(),
		});
	}

	// Extract messages, notes, etc. from tiles
	let mut messages: Vec<MessageLayout> = Vec::new();
	let mut activations: Vec<ActivationLayout> = Vec::new();
	let mut destroys: Vec<DestroyLayout> = Vec::new();
	let mut notes: Vec<NoteLayout> = Vec::new();
	let mut dividers: Vec<DividerLayout> = Vec::new();
	let mut delays: Vec<DelayLayout> = Vec::new();
	let mut refs: Vec<RefLayout> = Vec::new();
	let mut fragments: Vec<FragmentLayout> = Vec::new();
	let mut fragment_stack: Vec<(f64, FragmentKind, String, Vec<(f64, String)>)> = Vec::new();

	// Compute total width from Real values
	let mut total_min_x = f64::MAX;
	let mut total_max_x = f64::MIN;
	for living in &livings {
		let b = get_x(living.pos_b);
		let d = get_x(living.pos_d);
		if b < total_min_x {
			total_min_x = b;
		}
		if d > total_max_x {
			total_max_x = d;
		}
	}
	if total_min_x == f64::MAX {
		total_min_x = 0.0;
	}
	if total_max_x == f64::MIN {
		total_max_x = 0.0;
	}
	// Extend extents to account for self-messages and notes that extend
	// beyond participant positions (e.g., single-participant self-messages).
	for tile in &tiles {
		match tile {
			TeozTile::SelfMessage {
				participant_idx, text_width, direction, active_level, ..
			} => {
				let cx = get_x(livings[*participant_idx].pos_c);
				let tm = TextMetrics::new(7.0, 7.0, 1.0, *text_width, tp.msg_line_height);
				let self_ext = rose::self_arrow_preferred_size(&tm).width
					+ live_thickness_width(*active_level);
				match direction {
					SeqDirection::LeftToRight => {
						let right = cx + self_ext;
						if right > total_max_x { total_max_x = right; }
					}
					SeqDirection::RightToLeft => {
						let left = cx - self_ext;
						if left < total_min_x { total_min_x = left; }
					}
				}
			}
			TeozTile::Note { participant_idx, is_left, width, .. } => {
				let cx = get_x(livings[*participant_idx].pos_c);
				if *is_left {
					let left = cx - *width - 5.0;
					if left < total_min_x { total_min_x = left; }
				} else {
					let right = cx + *width + 5.0;
					if right > total_max_x { total_max_x = right; }
				}
			}
			_ => {}
		}
	}
	let diagram_width = total_max_x - total_min_x;

	// Track activation state for ActivationLayout generation


	for tile in &tiles {
		match tile {
			TeozTile::Communication {
				from_idx,
				to_idx,
				text,
				text_lines,
				is_dashed,
				has_open_head,
				y,
				autonumber,
				circle_from,
				circle_to,
				..
			} => {
				let ty = y.unwrap_or(0.0);
				let from_x = get_x(livings[*from_idx].pos_c);
				let to_x = get_x(livings[*to_idx].pos_c);
				let is_left = to_x < from_x;
				messages.push(MessageLayout {
					from_x,
					to_x,
					y: ty,
					text: text.clone(),
					text_lines: text_lines.clone(),
					is_self: false,
					is_dashed: *is_dashed,
					is_left,
					has_open_head: *has_open_head,
					arrow_head: if *has_open_head { SeqArrowHead::Open } else { SeqArrowHead::Filled },
					autonumber: autonumber.clone(),
					source_line: None, // TODO: propagate from parser
					self_return_x: from_x,
					self_center_x: from_x,
					color: None,
					circle_from: *circle_from,
					circle_to: *circle_to,
				});
			}
			TeozTile::SelfMessage {
				participant_idx,
				text,
				text_lines,
				text_width,
				is_dashed,
				has_open_head,
				y,
				autonumber,
				direction,
				active_level,
				circle_from,
				circle_to,
				..
			} => {
				let ty = y.unwrap_or(0.0);
				let cx = get_x(livings[*participant_idx].pos_c);
				let is_left = *direction == SeqDirection::RightToLeft;
				let has_bar = *active_level > 0;

				// Compute self-message from_x/to_x/return_x accounting for
				// activation bar, matching Java's LivingParticipantBox logic.
				let (self_from_x, self_return_x, self_to_x) = if is_left {
					let act_left = if has_bar {
						cx - ACTIVATION_WIDTH / 2.0
					} else {
						cx
					};
					let outgoing_x = if has_bar { act_left } else { cx };
					let ret_x = act_left - 1.0;
					let to = act_left - SELF_MSG_WIDTH;
					(outgoing_x, ret_x, to)
				} else {
					let act_right = if has_bar {
						cx + ACTIVATION_WIDTH / 2.0
					} else {
						cx
					};
					let outgoing_x = if has_bar { act_right } else { cx };
					let ret_x = act_right + 1.0;
					let to = act_right + SELF_MSG_WIDTH;
					(outgoing_x, ret_x, to)
				};

				messages.push(MessageLayout {
					from_x: self_from_x,
					to_x: self_to_x,
					y: ty,
					text: text.clone(),
					text_lines: text_lines.clone(),
					is_self: true,
					is_dashed: *is_dashed,
					is_left,
					has_open_head: *has_open_head,
					arrow_head: if *has_open_head { SeqArrowHead::Open } else { SeqArrowHead::Filled },
					autonumber: autonumber.clone(),
					source_line: None, // TODO: propagate from parser
					self_return_x,
					self_center_x: cx,
					color: None,
					circle_from: *circle_from,
					circle_to: *circle_to,
				});
			}
			TeozTile::Note {
				participant_idx,
				text,
				is_left,
				width,
				height,
				y,
				..
			} => {
				let ty = y.unwrap_or(0.0);
				let cx = get_x(livings[*participant_idx].pos_c);
				let nx = if *is_left {
					cx - *width - 5.0
				} else {
					cx + 5.0
				};
				notes.push(NoteLayout {
					x: nx,
					y: ty,
					width: *width,
					layout_width: *width + 10.0,
					height: *height,
					text: text.clone(),
					is_left: *is_left,
					is_self_msg_note: false,
				});
			}
			TeozTile::NoteOver {
				participants,
				text,
				width,
				height,
				y,
			} => {
				let ty = y.unwrap_or(0.0);
				// Center the note between the first and last referenced participant
				let (left_x, right_x) = if participants.len() >= 2 {
					let idx0 = name_to_idx.get(&participants[0]).copied().unwrap_or(0);
					let idx1 = name_to_idx
						.get(participants.last().unwrap())
						.copied()
						.unwrap_or(0);
					(
						get_x(livings[idx0].pos_c),
						get_x(livings[idx1].pos_c),
					)
				} else if participants.len() == 1 {
					let idx0 = name_to_idx.get(&participants[0]).copied().unwrap_or(0);
					let cx = get_x(livings[idx0].pos_c);
					(cx - *width / 2.0, cx + *width / 2.0)
				} else {
					(total_min_x, total_max_x)
				};
				let center = (left_x + right_x) / 2.0;
				notes.push(NoteLayout {
					x: center - *width / 2.0,
					y: ty,
					width: *width,
					layout_width: *width + 10.0,
					height: *height,
					text: text.clone(),
					is_left: false,
					is_self_msg_note: false,
				});
			}
			TeozTile::Divider { text, y, .. } => {
				let ty = y.unwrap_or(0.0);
				dividers.push(DividerLayout {
					y: ty,
					x: total_min_x,
					width: diagram_width,
					text: text.clone(),
					height: 0.0,
					component_y: ty,
				});
			}
			TeozTile::Delay { text, height, y } => {
				let ty = y.unwrap_or(0.0);
				delays.push(DelayLayout {
					y: ty,
					height: *height,
					x: total_min_x,
					width: diagram_width,
					text: text.clone(),
					lifeline_break_y: ty,
				});
			}
			TeozTile::Ref {
				participants,
				label,
				height,
				y,
			} => {
				let ty = y.unwrap_or(0.0);
				let (rx, rw) = if participants.is_empty() {
					(total_min_x, diagram_width)
				} else {
					let idxs: Vec<usize> = participants
						.iter()
						.filter_map(|p| name_to_idx.get(p).copied())
						.collect();
					if idxs.is_empty() {
						(total_min_x, diagram_width)
					} else {
						let min_idx = *idxs.iter().min().unwrap();
						let max_idx = *idxs.iter().max().unwrap();
						let lx = get_x(livings[min_idx].pos_b);
						let rx = get_x(livings[max_idx].pos_d);
						(lx, rx - lx)
					}
				};
				refs.push(RefLayout {
					x: rx,
					y: ty,
					width: rw,
					height: *height,
					label: label.clone(),
				});
			}
			TeozTile::FragmentStart { kind, label, y, .. } => {
				let ty = y.unwrap_or(0.0);
				fragment_stack.push((ty, kind.clone(), label.clone(), Vec::new()));
			}
			TeozTile::FragmentSeparator { label, y, .. } => {
				let ty = y.unwrap_or(0.0);
				if let Some(entry) = fragment_stack.last_mut() {
					entry.3.push((ty, label.clone()));
				}
			}
			TeozTile::FragmentEnd { y, .. } => {
				let ty = y.unwrap_or(0.0);
				if let Some((y_start, kind, label, separators)) = fragment_stack.pop() {
					fragments.push(FragmentLayout {
						kind,
						label,
						x: total_min_x,
						y: y_start,
						width: diagram_width,
						height: ty - y_start,
						separators,
					});
				}
			}
			_ => {}
		}
	}

	// Build activation bars from the event stream.
	// Re-scan events to track activate/deactivate pairs.
	{
		let mut act_state: HashMap<String, Vec<(f64, usize)>> = HashMap::new();
		let mut tile_idx = 0;
		for event in &sd.events {
			match event {
				SeqEvent::Activate(name) => {
					let ty = tiles
						.get(tile_idx)
						.and_then(|t| t.get_y())
						.unwrap_or(0.0);
					let stack = act_state.entry(name.clone()).or_default();
					let level = stack.len() + 1; // 1-based
					stack.push((ty, level));
				}
				SeqEvent::Deactivate(name) => {
					let ty = tiles
						.get(tile_idx)
						.and_then(|t| t.get_y())
						.unwrap_or(0.0);
					if let Some(stack) = act_state.get_mut(name) {
						if let Some((y_start, level)) = stack.pop() {
							let idx = name_to_idx.get(name).copied().unwrap_or(0);
							let cx = get_x(livings[idx].pos_c);
							let x = cx - ACTIVATION_WIDTH / 2.0
								+ (level - 1) as f64 * (ACTIVATION_WIDTH / 2.0);
							activations.push(ActivationLayout {
								participant: name.clone(),
								x,
								y_start,
								y_end: ty,
								level,
							});
						}
					}
				}
				SeqEvent::Destroy(name) => {
					let ty = tiles
						.get(tile_idx)
						.and_then(|t| t.get_y())
						.unwrap_or(0.0);
					let idx = name_to_idx.get(name).copied().unwrap_or(0);
					let cx = get_x(livings[idx].pos_c);
					destroys.push(DestroyLayout { x: cx, y: ty });
					// Close any open activations
					if let Some(stack) = act_state.get_mut(name) {
						while let Some((y_start, level)) = stack.pop() {
							let x = cx - ACTIVATION_WIDTH / 2.0
								+ (level - 1) as f64 * (ACTIVATION_WIDTH / 2.0);
							activations.push(ActivationLayout {
								participant: name.clone(),
								x,
								y_start,
								y_end: ty,
								level,
							});
						}
					}
				}
				_ => {}
			}
			tile_idx += 1;
		}
		// Close any unclosed activations at the lifeline bottom
		for (name, stack) in act_state.drain() {
			let idx = name_to_idx.get(&name).copied().unwrap_or(0);
			let cx = get_x(livings[idx].pos_c);
			for (y_start, level) in stack {
				let x = cx - ACTIVATION_WIDTH / 2.0
					+ (level - 1) as f64 * (ACTIVATION_WIDTH / 2.0);
				activations.push(ActivationLayout {
					participant: name.clone(),
					x,
					y_start,
					y_end: lifeline_bottom,
					level,
				});
			}
		}
	}

	// Java: TextBlock width = (maxX - minX) + 10, final = + margin(5+5) = + 20
	let total_width = diagram_width + 2.0 * DOC_MARGIN_X;
	// Java height chain:
	//   getPreferredHeight  = finalY + 10          (PlayingSpace bottom padding)
	//   bodyHeight          = preferred + factor*headHeight  (footbox adds 2nd head)
	//   calculateDimension  = bodyHeight + 10       (outer TextBlock wrapper)
	//   SVG viewport        = dimension + 10        (doc margin: UTranslate(5,5))
	//
	// Combined: startingY + sum_tiles + 10 + factor*headHeight + 10 + 10
	// Our lifeline_bottom already = startingY + headHeight + sum_tiles,
	// so: total = lifeline_bottom + (factor-1)*headHeight + 30
	let show_footbox = !sd.hide_footbox;
	let factor = if show_footbox { 2 } else { 1 };
	// Java height chain (no footbox, factor=1):
	//   startingY(8) + sum_tiles → finalY
	//   getPreferredHeight  = finalY + 10
	//   body.height          = preferred + 1*headHeight
	//   textBlock.height     = body.height + 10
	//   finalDim.height      = textBlock.height + margin(5+5)
	//   SVG viewport         = (int)(finalDim.height + 1)
	// Combined: sum + head + 39 (with our STARTING_Y=10 → sum + head + 38 + 1)
	// lifeline_bottom = STARTING_Y(10) + head + sum
	// total = lifeline_bottom + (factor-1)*head + 28
	let total_height =
		lifeline_bottom + (factor - 1) as f64 * max_preferred_height + 27.0;
	log::debug!("teoz_layout: total_width={total_width:.4} total_height={total_height:.4} lifeline_bottom={lifeline_bottom:.4} max_preferred_height={max_preferred_height:.4}");

	Ok(SeqLayout {
		participants: part_layouts,
		messages,
		activations,
		destroys,
		notes,
		groups: Vec::new(),
		fragments,
		dividers,
		delays,
		refs,
		autonumber_enabled,
		autonumber_start,
		lifeline_top: STARTING_Y + max_preferred_height,
		lifeline_bottom,
		total_width,
		total_height,
	})
}

// ── Text wrapping helper (copied from Puma) ──────────────────────────────────

fn wrap_text_to_width(
	text: &str,
	max_width: f64,
	font_family: &str,
	font_size: f64,
) -> Vec<String> {
	let full_w = font_metrics::text_width(text, font_family, font_size, false, false);
	if full_w <= max_width {
		return vec![text.to_string()];
	}
	let mut lines = Vec::new();
	let mut current = String::new();
	for word in text.split_whitespace() {
		let candidate = if current.is_empty() {
			word.to_string()
		} else {
			format!("{current} {word}")
		};
		let w = font_metrics::text_width(&candidate, font_family, font_size, false, false);
		if w > max_width && !current.is_empty() {
			lines.push(current);
			current = word.to_string();
		} else {
			current = candidate;
		}
	}
	if !current.is_empty() {
		lines.push(current);
	}
	if lines.is_empty() {
		vec![text.to_string()]
	} else {
		lines
	}
}
