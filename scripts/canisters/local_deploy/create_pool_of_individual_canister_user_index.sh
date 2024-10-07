# !/bin/bash



# Specify the path to your Wasm.gz file
wasm=".dfx/local/canisters/individual_user_template/individual_user_template.wasm.gz"

char=$(hexdump -ve '1/1 "%.2x"' "$wasm")

# Escape special characters in the hexadecimal string
char_escaped=$(printf "%s" "$char" | sed 's/../\\&/g')

# Create a shell script with the escaped hexadecimal string
printf "(\"v1.0.0\", blob \"%s\")"  "$char_escaped" > argument
dfx ledger fabricate-cycles --cycles 20000000000000000 --canister user_index
dfx canister call user_index create_pool_of_individual_user_available_canisters --argument-file argument
