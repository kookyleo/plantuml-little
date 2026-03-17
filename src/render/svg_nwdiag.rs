use crate::klimt::svg::SvgGraphic;
use crate::layout::nwdiag::{
    NwdiagConnectorLayout, NwdiagLayout, NwdiagNetworkLayout, NwdiagServerLayout,
};
use crate::model::nwdiag::NwdiagDiagram;
use crate::render::svg::{write_svg_root_bg, write_bg_rect};
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;

const LINE_HEIGHT: f64 = 16.0;
use crate::skin::rose::{BORDER_COLOR, DIVIDER_COLOR, ENTITY_BG, TEXT_COLOR};
const NETWORK_FILL: &str = "#F5F5F5";
const NETWORK_BORDER: &str = "#A0A0A0";

pub fn render_nwdiag(
    diagram: &NwdiagDiagram,
    layout: &NwdiagLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    let bg = skin.get_or("backgroundcolor", "#FFFFFF");
    write_svg_root_bg(&mut buf, layout.width, layout.height, "NWDIAG", bg);
    buf.push_str("<defs/><g>");
    write_bg_rect(&mut buf, layout.width, layout.height, bg);

    let mut sg = SvgGraphic::new(0, 1.0);

    if let Some(title) = &diagram.title {
        let mut tmp = String::new();
        render_creole_text(
            &mut tmp,
            title,
            layout.width / 2.0,
            20.0,
            LINE_HEIGHT,
            skin.font_color("nwdiag", TEXT_COLOR),
            Some("middle"),
            r#"font-size="14" font-weight="bold""#,
        );
        sg.push_raw(&tmp);
    }

    for connector in &layout.connectors {
        render_connector(&mut sg, connector);
    }
    for network in &layout.networks {
        render_network(&mut sg, network, skin);
    }
    for server in &layout.servers {
        render_server(&mut sg, server, skin);
    }

    buf.push_str(sg.body());
    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_connector(sg: &mut SvgGraphic, connector: &NwdiagConnectorLayout) {
    sg.set_stroke_color(Some(DIVIDER_COLOR));
    sg.set_stroke_width(0.5, Some((4.0, 4.0)));
    sg.svg_line(connector.x, connector.y1, connector.x, connector.y2, 0.0);
}

fn render_network(sg: &mut SvgGraphic, network: &NwdiagNetworkLayout, skin: &SkinParams) {
    let fill = network
        .color
        .as_deref()
        .unwrap_or_else(|| skin.background_color("nwdiag", NETWORK_FILL));
    let border = skin.border_color("nwdiag", NETWORK_BORDER);
    let font = skin.font_color("nwdiag", TEXT_COLOR);

    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(network.x, network.y, network.width, network.height, 8.0, 8.0, 0.0);

    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &network.name,
        network.x + 12.0,
        network.y + 22.0,
        LINE_HEIGHT,
        font,
        None,
        r#"font-size="14" font-weight="bold""#,
    );
    sg.push_raw(&tmp);

    if let Some(address) = &network.address {
        tmp.clear();
        render_creole_text(
            &mut tmp,
            address,
            network.x + 12.0,
            network.y + 40.0,
            LINE_HEIGHT,
            font,
            None,
            r#"font-size="11""#,
        );
        sg.push_raw(&tmp);
    }
}

fn render_server(sg: &mut SvgGraphic, server: &NwdiagServerLayout, skin: &SkinParams) {
    let fill = skin.background_color("server", ENTITY_BG);
    let border = skin.border_color("server", BORDER_COLOR);
    let font = skin.font_color("server", TEXT_COLOR);

    sg.set_fill_color(fill);
    sg.set_stroke_color(Some(border));
    sg.set_stroke_width(0.5, None);
    sg.svg_rectangle(server.x, server.y, server.width, server.height, 4.0, 4.0, 0.0);

    let mut tmp = String::new();
    render_creole_text(
        &mut tmp,
        &server.label,
        server.x + server.width / 2.0,
        server.y + 18.0,
        LINE_HEIGHT,
        font,
        Some("middle"),
        r#"font-size="12""#,
    );
    sg.push_raw(&tmp);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::nwdiag::{
        NwdiagConnectorLayout, NwdiagLayout, NwdiagNetworkLayout, NwdiagServerLayout,
    };
    use crate::style::SkinParams;

    fn sample_layout() -> (NwdiagDiagram, NwdiagLayout) {
        let diagram = NwdiagDiagram {
            title: Some("Infra".to_string()),
            networks: vec![],
        };
        let layout = NwdiagLayout {
            title_height: 28.0,
            networks: vec![NwdiagNetworkLayout {
                name: "dmz".to_string(),
                address: Some("10.0.0.0/24".to_string()),
                color: Some("#E8F4FF".to_string()),
                x: 20.0,
                y: 48.0,
                width: 360.0,
                height: 76.0,
            }],
            servers: vec![NwdiagServerLayout {
                network_name: "dmz".to_string(),
                name: "web01".to_string(),
                label: "web01\n10.0.0.10".to_string(),
                x: 220.0,
                y: 64.0,
                width: 100.0,
                height: 42.0,
            }],
            connectors: vec![NwdiagConnectorLayout {
                x: 270.0,
                y1: 84.0,
                y2: 150.0,
            }],
            width: 420.0,
            height: 180.0,
        };
        (diagram, layout)
    }

    #[test]
    fn render_contains_network_and_server() {
        let (diagram, layout) = sample_layout();
        let svg = render_nwdiag(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("dmz"));
        assert!(svg.contains("web01"));
    }

    #[test]
    fn render_contains_connector() {
        let (diagram, layout) = sample_layout();
        let svg = render_nwdiag(&diagram, &layout, &SkinParams::default()).unwrap();
        assert!(svg.contains("stroke-dasharray:4,4;"));
    }
}
