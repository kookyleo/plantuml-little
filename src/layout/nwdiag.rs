use std::collections::HashMap;

use crate::model::nwdiag::{NwdiagDiagram, ServerRef};
use crate::Result;

#[derive(Debug, Clone)]
pub struct NwdiagLayout {
    pub title_height: f64,
    pub networks: Vec<NwdiagNetworkLayout>,
    pub servers: Vec<NwdiagServerLayout>,
    pub connectors: Vec<NwdiagConnectorLayout>,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct NwdiagNetworkLayout {
    pub name: String,
    pub address: Option<String>,
    pub color: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct NwdiagServerLayout {
    pub network_name: String,
    pub name: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone)]
pub struct NwdiagConnectorLayout {
    pub x: f64,
    pub y1: f64,
    pub y2: f64,
}

const MARGIN: f64 = 20.0;
const TITLE_HEIGHT: f64 = 28.0;
const NETWORK_LABEL_WIDTH: f64 = 180.0;
const COLUMN_WIDTH: f64 = 150.0;
const BAND_HEIGHT: f64 = 76.0;
const BAND_GAP: f64 = 18.0;
const SERVER_MIN_WIDTH: f64 = 92.0;
const SERVER_MIN_HEIGHT: f64 = 34.0;
const CHAR_WIDTH: f64 = 7.2;
const LINE_HEIGHT: f64 = 15.0;
const PAD_H: f64 = 8.0;
const PAD_V: f64 = 6.0;

fn server_label(server: &ServerRef) -> String {
    let mut parts = vec![server.name.clone()];
    if let Some(address) = &server.address {
        parts.push(address.clone());
    }
    if let Some(description) = &server.description {
        parts.push(description.clone());
    }
    parts.join("\n")
}

fn server_size(server: &ServerRef) -> (f64, f64) {
    let label = server_label(server);
    let lines: Vec<&str> = label.lines().collect();
    let max_line = lines
        .iter()
        .map(|line| line.chars().count() as f64 * CHAR_WIDTH)
        .fold(0.0_f64, f64::max);
    let width = (max_line + 2.0 * PAD_H).max(SERVER_MIN_WIDTH);
    let height = (lines.len().max(1) as f64 * LINE_HEIGHT + 2.0 * PAD_V).max(SERVER_MIN_HEIGHT);
    (width, height)
}

fn ordered_server_names(diagram: &NwdiagDiagram) -> Vec<String> {
    let mut seen = HashMap::<String, usize>::new();
    let mut names = Vec::new();
    for network in &diagram.networks {
        for server in &network.servers {
            if !seen.contains_key(&server.name) {
                seen.insert(server.name.clone(), names.len());
                names.push(server.name.clone());
            }
        }
    }
    names
}

pub fn layout_nwdiag(diagram: &NwdiagDiagram) -> Result<NwdiagLayout> {
    let server_names = ordered_server_names(diagram);
    let title_height = if diagram.title.is_some() {
        TITLE_HEIGHT
    } else {
        0.0
    };
    let grid_width = server_names.len().max(1) as f64 * COLUMN_WIDTH;
    let band_width = NETWORK_LABEL_WIDTH + grid_width;

    let mut networks = Vec::new();
    let mut servers = Vec::new();
    let mut positions_by_server: HashMap<String, Vec<(f64, f64)>> = HashMap::new();

    for (row, network) in diagram.networks.iter().enumerate() {
        let y = MARGIN + title_height + row as f64 * (BAND_HEIGHT + BAND_GAP);
        let band = NwdiagNetworkLayout {
            name: network.name.clone(),
            address: network.address.clone(),
            color: network.color.clone(),
            x: MARGIN,
            y,
            width: band_width,
            height: BAND_HEIGHT,
        };
        networks.push(band.clone());

        for server in &network.servers {
            let col = server_names
                .iter()
                .position(|name| name == &server.name)
                .unwrap_or(0);
            let (width, height) = server_size(server);
            let cell_x = MARGIN + NETWORK_LABEL_WIDTH + col as f64 * COLUMN_WIDTH;
            let x = cell_x + (COLUMN_WIDTH - width) / 2.0;
            let y = band.y + (band.height - height) / 2.0;
            let label = server_label(server);

            servers.push(NwdiagServerLayout {
                network_name: network.name.clone(),
                name: server.name.clone(),
                label,
                x,
                y,
                width,
                height,
            });
            positions_by_server
                .entry(server.name.clone())
                .or_default()
                .push((x + width / 2.0, y + height / 2.0));
        }
    }

    let mut connectors = Vec::new();
    for positions in positions_by_server.values() {
        if positions.len() < 2 {
            continue;
        }
        let x = positions[0].0;
        let y1 = positions.first().map_or(0.0, |(_, y)| *y);
        let y2 = positions.last().map_or(y1, |(_, y)| *y);
        connectors.push(NwdiagConnectorLayout { x, y1, y2 });
    }

    let width = MARGIN + band_width + MARGIN;
    let height = if diagram.networks.is_empty() {
        MARGIN + title_height + BAND_HEIGHT + MARGIN
    } else {
        networks
            .last()
            .map_or(MARGIN + title_height + BAND_HEIGHT + MARGIN, |band| {
                band.y + band.height + MARGIN
            })
    };

    Ok(NwdiagLayout {
        title_height,
        networks,
        servers,
        connectors,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::nwdiag::{Network, NwdiagDiagram, ServerRef};

    fn sample_diagram() -> NwdiagDiagram {
        NwdiagDiagram {
            title: Some("Network".to_string()),
            networks: vec![
                Network {
                    name: "dmz".to_string(),
                    address: Some("10.0.0.0/24".to_string()),
                    color: Some("#E8F4FF".to_string()),
                    servers: vec![
                        ServerRef {
                            name: "web01".to_string(),
                            address: Some("10.0.0.10".to_string()),
                            description: Some("frontend".to_string()),
                        },
                        ServerRef {
                            name: "db01".to_string(),
                            address: None,
                            description: None,
                        },
                    ],
                },
                Network {
                    name: "lan".to_string(),
                    address: None,
                    color: None,
                    servers: vec![
                        ServerRef {
                            name: "web01".to_string(),
                            address: None,
                            description: Some("app".to_string()),
                        },
                        ServerRef {
                            name: "app01".to_string(),
                            address: None,
                            description: None,
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn layout_contains_network_rows() {
        let layout = layout_nwdiag(&sample_diagram()).unwrap();
        assert_eq!(layout.networks.len(), 2);
        assert_eq!(layout.servers.len(), 4);
    }

    #[test]
    fn repeated_servers_share_same_column() {
        let layout = layout_nwdiag(&sample_diagram()).unwrap();
        let web_positions: Vec<f64> = layout
            .servers
            .iter()
            .filter(|server| server.name == "web01")
            .map(|server| server.x)
            .collect();
        assert_eq!(web_positions.len(), 2);
        assert!((web_positions[0] - web_positions[1]).abs() < 0.1);
    }

    #[test]
    fn repeated_servers_get_connector() {
        let layout = layout_nwdiag(&sample_diagram()).unwrap();
        assert_eq!(layout.connectors.len(), 1);
        assert!(layout.connectors[0].y2 > layout.connectors[0].y1);
    }
}
