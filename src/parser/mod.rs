pub mod activity;
pub mod class;
pub mod common;
pub mod component;
pub mod creole;
pub mod ditaa;
pub mod dot;
pub mod erd;
pub mod gantt;
pub mod json_diagram;
pub mod mindmap;
pub mod nwdiag;
pub mod salt;
pub mod sequence;
pub mod state;
pub mod timing;
pub mod usecase;
pub mod wbs;
pub mod yaml;

use crate::model::diagram::ClassDiagram;
use crate::model::Diagram;
use crate::Result;

pub fn parse(source: &str) -> Result<Diagram> {
    // First check for specialized @start tags
    let tag_hint = common::detect_start_tag(source);
    if let Some(hint) = tag_hint {
        return match hint {
            DiagramHint::Erd => {
                let ed = erd::parse_erd_diagram(source)?;
                Ok(Diagram::Erd(ed))
            }
            DiagramHint::Gantt => {
                let gd = gantt::parse_gantt_diagram(source)?;
                Ok(Diagram::Gantt(gd))
            }
            DiagramHint::Ditaa => {
                let dd = ditaa::parse_ditaa(source)?;
                Ok(Diagram::Ditaa(dd))
            }
            DiagramHint::Json => {
                let jd = json_diagram::parse_json_diagram(source)?;
                Ok(Diagram::Json(jd))
            }
            DiagramHint::Mindmap => {
                let md = mindmap::parse_mindmap_diagram(source)?;
                Ok(Diagram::Mindmap(md))
            }
            DiagramHint::Nwdiag => {
                let nd = nwdiag::parse_nwdiag_diagram(source)?;
                Ok(Diagram::Nwdiag(nd))
            }
            DiagramHint::Salt => {
                let sd = salt::parse_salt_diagram(source)?;
                Ok(Diagram::Salt(sd))
            }
            DiagramHint::Wbs => {
                let wd = wbs::parse_wbs_diagram(source)?;
                Ok(Diagram::Wbs(wd))
            }
            DiagramHint::Yaml => {
                let yd = yaml::parse_yaml_diagram(source)?;
                Ok(Diagram::Yaml(yd))
            }
            DiagramHint::Dot => {
                let block = common::extract_block(source).unwrap_or_default();
                let ds = dot::parse_dot_source(&block)?;
                Ok(Diagram::Dot(crate::model::dot::DotDiagram { source: ds }))
            }
            _ => unreachable!(),
        };
    }

    // For @startuml, use heuristic detection
    let content = common::extract_block(source);
    let body = content.as_deref().unwrap_or(source);
    let dtype = common::detect_diagram_type(body);

    match dtype {
        DiagramHint::Class => {
            let cd = class::parse_class_diagram(source)?;
            Ok(Diagram::Class(cd))
        }
        DiagramHint::Sequence => {
            let sd = sequence::parse_sequence_diagram(source)?;
            Ok(Diagram::Sequence(sd))
        }
        DiagramHint::Activity => {
            let ad = activity::parse_activity_diagram(source)?;
            Ok(Diagram::Activity(ad))
        }
        DiagramHint::State => {
            let sd = state::parse_state_diagram(source)?;
            Ok(Diagram::State(sd))
        }
        DiagramHint::UseCase => {
            let ud = usecase::parse_usecase_diagram(source)?;
            Ok(Diagram::UseCase(ud))
        }
        DiagramHint::Component => {
            let cd = component::parse_component_diagram(source)?;
            Ok(Diagram::Component(cd))
        }
        DiagramHint::Timing => {
            let td = timing::parse_timing_diagram(source)?;
            Ok(Diagram::Timing(td))
        }
        DiagramHint::Salt => {
            let sd = salt::parse_salt_diagram(source)?;
            Ok(Diagram::Salt(sd))
        }
        DiagramHint::Unknown(t) => {
            // Meta-only diagrams default to empty class diagram, matching Java
            // PlantUML which produces data-diagram-type="CLASS" for these.
            if !common::has_meaningful_uml_content(body) && !common::parse_meta(source).is_empty() {
                return Ok(Diagram::Class(ClassDiagram {
                    entities: Vec::new(),
                    links: Vec::new(),
                    groups: Vec::new(),
                    direction: Default::default(),
                    direction_explicit: false,
                    notes: Vec::new(),
                    hide_show_rules: Vec::new(),
                    stereotype_backgrounds: Default::default(),
                }));
            }
            Err(crate::Error::UnsupportedDiagram(t))
        }
        // These should be handled by start tag detection above
        DiagramHint::Ditaa
        | DiagramHint::Erd
        | DiagramHint::Gantt
        | DiagramHint::Json
        | DiagramHint::Mindmap
        | DiagramHint::Nwdiag
        | DiagramHint::Wbs
        | DiagramHint::Yaml
        | DiagramHint::Dot => Err(crate::Error::UnsupportedDiagram(format!("{dtype:?}"))),
    }
}

/// Internal diagram type hint
#[derive(Debug)]
pub enum DiagramHint {
    Class,
    Sequence,
    Activity,
    State,
    Component,
    Ditaa,
    Erd,
    Gantt,
    Json,
    Mindmap,
    Nwdiag,
    Salt,
    Timing,
    Wbs,
    Yaml,
    Dot,
    UseCase,
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_meta_only_uml_as_empty_class_diagram() {
        let src = "@startuml\ntitle\nOnly meta\nend title\n@enduml\n";
        let diagram = parse(src).expect("parse failed");
        match diagram {
            Diagram::Class(cd) => {
                assert!(cd.entities.is_empty());
                assert!(cd.links.is_empty());
                assert!(cd.notes.is_empty());
            }
            other => panic!("expected empty class fallback, got {:?}", other),
        }
    }
}
