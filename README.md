**Monday Claim CLI**

A command-line interface tool for managing items on a [Monday.com](https://monday.com/) board. This tool allows you to query existing items and add new items with specific activity types.

**Features**

- **Query items**: Fetch and display items from your [Monday.com](https://monday.com/) board with pretty table formatting
- **Add items**: Create new items with configurable activity types, dates, and details
- **Group management**: Automatically organizes items by year-based groups
- **Activity types**: Support for multiple activity types with human-readable names

**Installation**

1. **Install Rust**: Make sure you have Rust installed on your system.
2. **Clone the repository**:

  ```bash
    git clone &lt;your-repository-url&gt;
    cd monday-claim
  ```

3. **Build the project**:

   ```bash
    cargo build --release
   ```

**Configuration**

Create a config.toml file with your [Monday.com](https://monday.com/) API credentials:

   ```bash
    api_key = "your_monday_api_key_here"
    board_id = "your_board_id_here"
    user_id = "your_user_id_here"
   ```

**Getting API Credentials**

1. **API Key**: Go to [Monday.com](https://monday.com/) → Your profile → Admin → API → Generate new API token
2. **Board ID**: Open your board in a web browser and copy the ID from the URL
3. **User ID**: Can be found in your [Monday.com](https://monday.com/) account settings or via API

**Usage**

**Query Items**

View items from your board:

   ```bash
    cargo run -- --config config.toml query
   ```

Limit the number of items displayed:

   ```bash
    cargo run -- --config config.toml query --limit 5
   ```

**Add New Item**

Add a new item to the board:

   ```bash
    cargo run -- --config config.toml add \\
    \--year "2025" \\
    \--name "Your Name" \\
    \--activity "billable" \\
    \--date "2025-09-05" \\
    \--client "Client Name" \\
    \--wi "Project Code" \\
    \--hours "8"
   ```

**Short Options**

You can also use short options:

   ```bash
    cargo run -- --config config.toml add \\
    \-y "2025" \\
    \-n "Your Name" \\
    \-a "billable" \\
    \-d "2025-09-05" \\
    \-c "Client Name" \\
    \-w "Project Code" \\
    \-H "8"
   ```
   
**Activity Types**

The following activity types are supported:

| Activity | Value | Description |
| --- | --- | --- |
| vacation | 0   | Vacation time |
| billable | 1   | Billable work |
| holding | 2   | Holding/placeholder |
| education | 3   | Training/education |
| work_reduction | 4   | Reduced work hours |
| tbd | 5   | To be determined |
| holiday | 6   | Public holiday |
| ""  | 7   | Empty (not used) |
| illness | 8   | Sick leave |

**Output Format**

The query command displays:

- A table of groups with their IDs and titles
- A table of items with their details and column values
- Group information showing which group each item belongs to

**Error Handling**

The tool provides detailed error messages for:

- API authentication issues
- Invalid parameters
- [Monday.com](https://monday.com/) API errors
- Network connectivity problems

**Dependencies**

- **reqwest**: HTTP client for API requests
- **serde**: JSON serialization/deserialization
- **clap**: Command-line argument parsing
- **prettytable**: Formatting output as tables
- **tokio**: Async runtime for HTTP requests
- **anyhow**: Error handling

**License**

This project is licensed under the MIT License.

**Contributing**

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

**Support**

For issues and questions, please open an issue on the GitHub repository.
