# bridge

Configure and execute CRUD operations on certain resources of a remote Directus backend without needing to update the client's code.

## Examples
```bash
./target/release/bridge ls # without previously syncing config from server
# 
./target/release/bridge sync 'APP_NAME'
# $ {"app_name":"APP_NAME","base_path":"/API_PATH","config_sync_field_name":"FOO","config_sync_resource_name":"BAR","origin":"https://YOUR_DOMAIN","resources":{"APPLES":{"ops":{"create":true,"delete":true,"read":true,"update":true}}}}
./target/release/bridge ls # after syncing config from server
# $ APPLES
./target/release/bridge inspect 'APPLES'
# $ {"ops":{"create":true,"delete":true,"read":true,"update":true}}
./target/release/bridge read 'APPLES' # for all records (assuming there are already some items saved for this resource on the server)
# $ [{"id": 1, "color": "green", "grams": 250},{"id": 2, "color": "red", "grams": 425}]
./target/release/bridge read 'APPLES' 1
# $ {"id": 1, "color": "green", "grams": 250}
./target/release/bridge create 'APPLES' '{"color": "red", "grams": 400}'
# $ created successfully
./target/release/bridge update 'APPLES' 1 '{"color": "brown", "grams": 0}'
# $ updated successfully
./target/release/bridge delete 'APPLES' 1
# $ deleted successfully
./target/release/bridge read 'APPLES'
# $ [{"id": 1, "color": "brown", "grams": 0},{"id": 2, "color": "red", "grams": 425},{"id": 3, "color": "red", "grams": 400}]
./target/release/bridge read 'ZEBRAS' # assuming that there is no resource available for this app on the server
# $ error: resource not available
```

## Installation
1. Rename `.config.example.json` to `.config.json` and update the values inside to match that of your specific Directus server setup
2. Run `cargo build --release` in the project root directory to build the `bridge` binary executable in the target/release/ directory
2. Add / make any changes to the config data on your server for this app instance
2. Run `./target/release/bridge sync > new_config.json && mv new_config.json .config.json` to pull that updated config data to a local file (currently, it's less error prone to save it to a separate file and then rename it into the .config.json afterward)

## Usage
- The top level commands are:
    * `sync` : update the local .config.json file from the corresponding Directus server (this requires certain fields in the .config.json file not be changed)
    * `ls` : list out the available resources for this app instance (this will be updated whenever the .config.json file is updated)
    * `inspect <RESOURCE>` : list out the available operations for the given resource for this app instance (this will be updated whenever the .config.json file is updated)
- The resource-specific commands that will be available depending on the configuration are (note that these operations will also be affected by the permission set on the server itself):
    * `create <RESOURCE> <DATA>` : create a new record for the given resource using the given JSON data
    * `read <RESOURCE>` : display all records for the given resource
        * `read <RESOURCE> <ID>` : display all records for the given resource with the given ID
    * `update <RESOURCE>` : update the record for the given resource with the given ID using the given JSON data
    * `delete <RESOURCE>` : delete the record for the given resource with the given ID
