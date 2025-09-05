use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct BoardStructureResponse {
    pub boards: Vec<Board>,
}

#[derive(Debug, Deserialize)]
pub struct Board {
    pub name: String,
    pub id: String,
    pub groups: Vec<Group>,
    pub items_page: ItemsPage,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ItemsPage {
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub group: GroupReference,
    pub column_values: Vec<ColumnValue>,
}

#[derive(Debug, Deserialize)]
pub struct GroupReference {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct ColumnValue {
    pub id: String,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateItemVariables {
    pub board_id: String,
    pub group_id: String,
    pub item_name: String,
    pub column_values: String,
}
