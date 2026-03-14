use std::fmt::Write;

use crate::layout::nwdiag::{
    NwdiagConnectorLayout, NwdiagLayout, NwdiagNetworkLayout, NwdiagServerLayout,
};
use crate::model::nwdiag::NwdiagDiagram;
use crate::render::svg::fmt_coord;
use crate::render::svg_richtext::render_creole_text;
use crate::style::SkinParams;
use crate::Result;
use super::svg::write_svg_root;

const LINE_HEIGHT: f64 = 16.0;
const NETWORK_FILL: &str = "#F5F5F5";
const NETWORK_BORDER: &str = "#A0A0A0";
const SERVER_FILL: &str = "#F1F1F1";
const SERVER_BORDER: &str = "#181818";
const TEXT_FILL: &str = "#000000";
const CONNECTOR_COLOR: &str = "#888888";

pub fn render_nwdiag(
    diagram: &NwdiagDiagram,
    layout: &NwdiagLayout,
    skin: &SkinParams,
) -> Result<String> {
    let mut buf = String::with_capacity(4096);

    write_svg_root(&mut buf, layout.width, layout.height, "NWDIAG");
    buf.push_str("<defs/><g>");

    if let Some(title) = &diagram.title {
        render_creole_text(
            &mut buf,
            title,
            layout.width / 2.0,
            20.0,
            LINE_HEIGHT,
            skin.font_color("nwdiag", TEXT_FILL),
            Some("middle"),
            r#"font-size="14" font-weight="bold""#,
        );
    }

    for connector in &layout.connectors {
        render_connector(&mut buf, connector);
    }
    for network in &layout.networks {
        render_network(&mut buf, network, skin);
    }
    for server in &layout.servers {
        render_server(&mut buf, server, skin);
    }

    buf.push_str("</g></svg>");
    Ok(buf)
}

fn render_connector(buf: &mut String, connector: &NwdiagConnectorLayout) {
    write!(
        buf,
        r#"<line style="stroke:{CONNECTOR_COLOR};stroke-width:0.5;stroke-dasharray:4,4;" x1="{cx}" x2="{cx}" y1="{}" y2="{}"/>"#,
        fmt_coord(connector.y1), fmt_coord(connector.y2),
        cx = fmt_coord(connector.x),
    )
    .unwrap();
    buf.push('\n');
}

fn render_network(buf: &mut String, network: &NwdiagNetworkLayout, skin: &SkinParams) {
    let fill = network
        .color
        .as_deref()
        .unwrap_or_else(|| skin.background_color("nwdiag", NETWORK_FILL));
    let border = skin.border_color("nwdiag", NETWORK_BORDER);
    let font = skin.font_color("nwdiag", TEXT_FILL);

    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="8" ry="8" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(network.height), fmt_coord(network.width), fmt_coord(network.x), fmt_coord(network.y),
    )
    .unwrap();
    buf.push('\n');

    render_creole_text(
        buf,
        &network.name,
        network.x + 12.0,
        network.y + 22.0,
        LINE_HEIGHT,
        font,
        None,
        r#"font-size="14" font-weight="bold""#,
    );
    if let Some(address) = &network.address {
        render_creole_text(
            buf,
            address,
            network.x + 12.0,
            network.y + 40.0,
            LINE_HEIGHT,
            font,
            None,
            r#"font-size="11""#,
        );
    }
}

fn render_server(buf: &mut String, server: &NwdiagServerLayout, skin: &SkinParams) {
    let fill = skin.background_color("server", SERVER_FILL);
    let border = skin.border_color("server", SERVER_BORDER);
    let font = skin.font_color("server", TEXT_FILL);

    write!(
        buf,
        r#"<rect fill="{fill}" height="{}" rx="4" ry="4" style="stroke:{border};stroke-width:0.5;" width="{}" x="{}" y="{}"/>"#,
        fmt_coord(server.height), fmt_coord(server.width), fmt_coord(server.x), fmt_coord(server.y),
    )
    .unwrap();
    buf.push('\n');

    render_creole_text(
        buf,
        &server.label,
        server.x + server.width / 2.0,
        server.y + 18.0,
        LINE_HEIGHT,
        font,
        Some("middle"),
        r#"font-size="12""#,
    );
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
