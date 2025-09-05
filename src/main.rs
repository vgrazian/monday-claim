mod config;
mod models;

use anyhow::Result;
use clap::{Arg, Command};
use config::Config;
use models::*;
use prettytable::{Table, row};
use reqwest::Client;
use serde_json::json;

const MONDAY_API_URL: &str = "https://api.monday.com/v2";

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("monday-claim")
        .version("1.0")
        .author("Valerio Graziani")
        .about("CLI tool for managing Monday.com board items")
        .arg(
            Arg::new("config")
                .short('C')
                .long("config")
                .value_name("FILE")
                .help("Path to config file")
                .required(true),
        )
        .subcommand(Command::new("query").about("Query board items").arg(
            Arg::new("limit")
                .short('l')
                .long("limit")
                .value_name("LIMIT")
                .help("Number of items to fetch (default: 10)")
                .default_value("10"),
        ))
        .subcommand(
            Command::new("add")
                .about("Add a new item to the board")
                .arg(
                    Arg::new("year")
                        .short('y')
                        .long("year")
                        .value_name("YEAR")
                        .help("Year for the group (e.g., 2024)")
                        .required(true),
                )
                .arg(
                    Arg::new("name")
                        .short('n')
                        .long("name")
                        .value_name("NAME")
                        .help("Item name")
                        .required(true),
                )
                .arg(
                    Arg::new("activity")
                        .short('a')
                        .long("activity")
                        .value_name("ACTIVITY")
                        .help("Activity type: vacation, billable, holding, education, work_reduction, tbd, holiday, illness")
                        .required(true),
                )
                .arg(
                    Arg::new("date")
                        .short('d')
                        .long("date")
                        .value_name("DATE")
                        .help("Date in YYYY-MM-DD format")
                        .required(true),
                )
                .arg(
                    Arg::new("client")
                        .short('c')
                        .long("client")
                        .value_name("CLIENT")
                        .help("Client name")
                        .required(true),
                )
                .arg(
                    Arg::new("wi")
                        .short('w')
                        .long("wi")
                        .value_name("WORK_ITEM")
                        .help("Work item code")
                        .required(true),
                )
                .arg(
                    Arg::new("hours")
                        .short('H')
                        .long("hours")
                        .value_name("HOURS")
                        .help("Number of hours")
                        .required(true),
                ),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config").unwrap();
    let config = Config::from_file(config_path)?;

    let client = Client::new();

    match matches.subcommand() {
        Some(("query", query_matches)) => {
            let limit = query_matches.get_one::<String>("limit").unwrap();
            extract_board_items(&client, &config, limit).await?;
        }
        Some(("add", add_matches)) => {
            let year = add_matches.get_one::<String>("year").unwrap();
            let name = add_matches.get_one::<String>("name").unwrap();
            let activity = add_matches.get_one::<String>("activity").unwrap();
            let date = add_matches.get_one::<String>("date").unwrap();
            let client_name = add_matches.get_one::<String>("client").unwrap();
            let wi = add_matches.get_one::<String>("wi").unwrap();
            let hours = add_matches.get_one::<String>("hours").unwrap();

            add_board_item(
                &client,
                &config,
                year,
                name,
                activity,
                date,
                client_name,
                wi,
                hours,
            )
            .await?;
        }
        _ => {
            println!("No subcommand provided. Use --help for usage information.");
        }
    }

    Ok(())
}

async fn extract_board_items(client: &Client, config: &Config, limit: &str) -> Result<()> {
    // Build the GraphQL query to get board structure including groups
    let board_structure_query = format!(
        r#"
        query GetBoardStructure {{
            boards(ids: "{}") {{
                name
                id
                groups {{
                    id
                    title
                }}
                items_page(limit: {}) {{
                    items {{
                        id
                        name
                        group {{
                            id
                        }}
                        column_values {{
                            id
                            value
                        }}
                    }}
                }}
            }}
        }}
        "#,
        config.board_id, limit
    );

    let request = GraphQLRequest {
        query: board_structure_query,
        variables: Some(serde_json::json!({})),
    };

    println!("Sending query to Monday.com API to get board structure...");

    let response_text = client
        .post(MONDAY_API_URL)
        .header("Authorization", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .text()
        .await?;

    println!("API Response received");

    // Parse the response
    match serde_json::from_str::<GraphQLResponse<models::BoardStructureResponse>>(&response_text) {
        Ok(response) => {
            if let Some(errors) = response.errors {
                for error in errors {
                    eprintln!("GraphQL Error: {}", error.message);
                }
                return Ok(());
            }

            if let Some(data) = response.data {
                if let Some(board) = data.boards.first() {
                    // Print groups information
                    print_groups_table(&board.groups);

                    // Print items information
                    print_items_table(&board.items_page.items, &board.groups);
                } else {
                    println!("No boards found with the specified ID.");
                }
            } else {
                println!("No data returned from API.");
            }
        }
        Err(e) => {
            eprintln!("Failed to parse response: {}", e);
            eprintln!("Raw response was: {}", response_text);

            // Try to parse as generic JSON to see what we got
            if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                eprintln!("Parsed as generic JSON: {:#?}", raw_json);
            }
        }
    }

    Ok(())
}

fn print_groups_table(groups: &[models::Group]) {
    if groups.is_empty() {
        println!("No groups found in the board.");
        return;
    }

    let mut table = Table::new();
    table.add_row(row!["Group ID", "Group Title"]);

    for group in groups {
        table.add_row(row![group.id, group.title]);
    }

    println!("Found {} groups:", groups.len());
    table.printstd();
    println!(); // Add empty line for separation
}

fn print_items_table(items: &[models::Item], groups: &[models::Group]) {
    if items.is_empty() {
        println!("No items found in the board.");
        return;
    }

    // Create a mapping from group ID to group title
    let group_mapping: std::collections::HashMap<&str, &str> = groups
        .iter()
        .map(|group| (group.id.as_str(), group.title.as_str()))
        .collect();

    // Collect all unique column IDs
    let mut column_ids = Vec::new();
    for item in items {
        for column in &item.column_values {
            if !column_ids.contains(&column.id) {
                column_ids.push(column.id.clone());
            }
        }
    }

    // Create table with headers
    let mut table = Table::new();

    // Build headers row
    let mut header_cells = vec![
        prettytable::Cell::new("ID"),
        prettytable::Cell::new("Name"),
        prettytable::Cell::new("Group"),
    ];
    for column_id in &column_ids {
        header_cells.push(prettytable::Cell::new(&format!("Column {}", column_id)));
    }
    table.add_row(prettytable::Row::new(header_cells));

    // Add data rows
    for item in items {
        let group_name = group_mapping
            .get(item.group.id.as_str())
            .unwrap_or(&"Unknown");

        let mut row_cells = vec![
            prettytable::Cell::new(&item.id),
            prettytable::Cell::new(&item.name),
            prettytable::Cell::new(group_name),
        ];

        for column_id in &column_ids {
            if let Some(column_value) = item.column_values.iter().find(|c| &c.id == column_id) {
                let display_value = match &column_value.value {
                    Some(value) => {
                        // Parse the JSON value if it's a JSON string, otherwise use as-is
                        if value.starts_with('{') || value.starts_with('[') {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(value) {
                                if let Some(text) = parsed.get("text").and_then(|v| v.as_str()) {
                                    text.to_string()
                                } else if let Some(date) =
                                    parsed.get("date").and_then(|v| v.as_str())
                                {
                                    date.to_string()
                                } else if let Some(ids) =
                                    parsed.get("ids").and_then(|v| v.as_array())
                                {
                                    if ids.is_empty() {
                                        "".to_string()
                                    } else {
                                        let id_strings: Vec<String> = ids
                                            .iter()
                                            .filter_map(|v| v.as_i64().map(|id| id.to_string()))
                                            .collect();
                                        id_strings.join(", ")
                                    }
                                } else if let Some(persons) =
                                    parsed.get("personsAndTeams").and_then(|v| v.as_array())
                                {
                                    if persons.is_empty() {
                                        "".to_string()
                                    } else {
                                        let person_ids: Vec<String> = persons
                                            .iter()
                                            .filter_map(|p| {
                                                p.get("id")
                                                    .and_then(|v| v.as_i64())
                                                    .map(|id| id.to_string())
                                            })
                                            .collect();
                                        person_ids.join(", ")
                                    }
                                } else if let Some(index) =
                                    parsed.get("index").and_then(|v| v.as_i64())
                                {
                                    index.to_string()
                                } else {
                                    // Fallback: just display the raw value
                                    value.clone()
                                }
                            } else {
                                value.clone()
                            }
                        } else {
                            // Remove quotes from string values
                            value.trim_matches('"').to_string()
                        }
                    }
                    None => "".to_string(),
                };

                row_cells.push(prettytable::Cell::new(&display_value));
            } else {
                row_cells.push(prettytable::Cell::new(""));
            }
        }

        table.add_row(prettytable::Row::new(row_cells));
    }

    println!("Found {} items:", items.len());
    table.printstd();
}

async fn add_board_item(
    client: &Client,
    config: &Config,
    year: &str,
    name: &str,
    activity: &str,
    date: &str,
    client_name: &str,
    wi: &str,
    hours: &str,
) -> Result<()> {
    // Map activity text to integer value
    let activity_value = match activity.to_lowercase().as_str() {
        "vacation" => 0,
        "billable" => 1,
        "holding" => 2,
        "education" => 3,
        "work_reduction" => 4,
        "tbd" => 5,
        "holiday" => 6,
        "" => 7,
        "illness" => 8,
        _ => {
            eprintln!("❌ Invalid activity type: {}", activity);
            eprintln!(
                "Valid activity types are: vacation, billable, holding, education, work_reduction, tbd, holiday, illness"
            );
            return Ok(());
        }
    };

    // First, get the board structure to find the group ID for the given year
    let board_structure_query = format!(
        r#"
        query GetBoardGroups {{
            boards(ids: "{}") {{
                groups {{
                    id
                    title
                }}
            }}
        }}
        "#,
        config.board_id
    );

    let request = GraphQLRequest {
        query: board_structure_query,
        variables: Some(serde_json::json!({})),
    };

    println!("Looking up group ID for year: {}", year);

    let response_text = client
        .post(MONDAY_API_URL)
        .header("Authorization", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .text()
        .await?;

    // Parse the response as raw JSON to extract groups
    let group_id = match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(response_value) => {
            if let Some(errors) = response_value.get("errors") {
                if let Some(error_array) = errors.as_array() {
                    for error in error_array {
                        if let Some(message) = error.get("message") {
                            eprintln!("GraphQL Error: {}", message);
                        }
                    }
                }
                return Ok(());
            }

            if let Some(data) = response_value.get("data") {
                if let Some(boards) = data.get("boards") {
                    if let Some(board_array) = boards.as_array() {
                        if let Some(board) = board_array.first() {
                            if let Some(groups) = board.get("groups") {
                                if let Some(groups_array) = groups.as_array() {
                                    // Find the group with the matching year
                                    let mut found_group_id = None;
                                    for group in groups_array {
                                        if let (Some(id), Some(title)) =
                                            (group.get("id"), group.get("title"))
                                        {
                                            if let (Some(id_str), Some(title_str)) =
                                                (id.as_str(), title.as_str())
                                            {
                                                if title_str == year {
                                                    found_group_id = Some(id_str.to_string());
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    match found_group_id {
                                        Some(id) => {
                                            println!("Found group ID: {} for year: {}", id, year);
                                            id
                                        }
                                        None => {
                                            eprintln!("❌ No group found with title: {}", year);
                                            eprintln!("Available groups:");
                                            for group in groups_array {
                                                if let (Some(id), Some(title)) =
                                                    (group.get("id"), group.get("title"))
                                                {
                                                    if let (Some(id_str), Some(title_str)) =
                                                        (id.as_str(), title.as_str())
                                                    {
                                                        eprintln!("  - {}: {}", title_str, id_str);
                                                    }
                                                }
                                            }
                                            return Ok(());
                                        }
                                    }
                                } else {
                                    eprintln!("❌ Groups is not an array");
                                    return Ok(());
                                }
                            } else {
                                eprintln!("❌ No groups field in board");
                                return Ok(());
                            }
                        } else {
                            eprintln!("❌ No boards found");
                            return Ok(());
                        }
                    } else {
                        eprintln!("❌ Boards is not an array");
                        return Ok(());
                    }
                } else {
                    eprintln!("❌ No boards field in data");
                    return Ok(());
                }
            } else {
                eprintln!("❌ No data in response");
                return Ok(());
            }
        }
        Err(e) => {
            eprintln!("Failed to parse group response: {}", e);
            eprintln!("Raw response was: {}", response_text);
            return Ok(());
        }
    };

    // Create column values JSON string using user_id from config
    let column_values = json!({
        "person": json!({
            "personsAndTeams": [{
                "id": config.user_id.parse::<i64>()?,
                "kind": "person"
            }]
        }),
        "status": json!({
            "index": activity_value
        }),
        "date4": json!({
            "date": date
        }),
        "text__1": client_name,
        "text8__1": wi,
        "numbers__1": hours
    })
    .to_string();

    let query = r#"
        mutation CreateItem($boardId: ID!, $groupId: String!, $itemName: String!, $columnValues: JSON!) {
            create_item(
                board_id: $boardId,
                group_id: $groupId,
                item_name: $itemName,
                column_values: $columnValues
            ) {
                id
                name
            }
        }
    "#;

    let variables = json!({
        "boardId": config.board_id,
        "groupId": group_id,
        "itemName": name,
        "columnValues": column_values
    });

    let request = GraphQLRequest {
        query: query.to_string(),
        variables: Some(variables),
    };

    println!("Creating new item: {}", name);
    println!("Activity: {} (index: {})", activity, activity_value);
    println!("Adding to group ID: {}", group_id);

    let response_text = client
        .post(MONDAY_API_URL)
        .header("Authorization", &config.api_key)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?
        .text()
        .await?;

    println!("Create item response: {}", response_text);

    // Parse the response
    match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(response_value) => {
            if let Some(errors) = response_value.get("errors") {
                if let Some(error_array) = errors.as_array() {
                    for error in error_array {
                        if let Some(message) = error.get("message") {
                            eprintln!("GraphQL Error: {}", message);
                        }
                    }
                }
                return Ok(());
            }

            if let Some(data) = response_value.get("data") {
                if let Some(create_item) = data.get("create_item") {
                    if let Some(id) = create_item.get("id") {
                        println!("✅ Item created successfully! ID: {}", id);
                    } else {
                        println!("✅ Item created successfully!");
                    }
                } else {
                    println!("❌ No create_item data in response");
                }
            } else {
                println!("❌ No data in response");
            }
        }
        Err(e) => {
            eprintln!("Failed to parse create response: {}", e);
            eprintln!("Raw response was: {}", response_text);
        }
    }

    Ok(())
}
