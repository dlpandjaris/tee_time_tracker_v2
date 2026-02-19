#!/bin/bash
# echo "First argument: $1"
# echo "All arguments: $@"
# echo "Number of arguments: $#"

# Ensure a URL was provided
if [ -z "$1" ]; then
  echo "Usage: $0 <base_url>"
  echo "Example: $0 http://localhost:8000"
  exit 1
fi

BASE_URL="$1"

echo "----------------------------------"

start=$(date +%s%3N)

# Run curl and capture response + timing
teetimes=$(curl -G "$BASE_URL/tee_times" \
  --data-urlencode "date=2026-02-21" \
  --data-urlencode "players=4" \
  --data-urlencode 'coords={%22min_lat%22:38.800618109910125,%22min_lon%22:-94.8499263107666,%22max_lat%22:38.85411044179456,%22max_lon%22:-94.6602404892334}')
#   --data-urlencode 'coords={"min_lat":38.686,"min_lon":-96.563,"max_lat":39.539,"max_lon":-93.528}')
#   --data-urlencode 'coords={min_lat:39.1618,min_lon:-94.899,max_lat:39.1752,max_lon:-94.8515}')
end=$(date +%s%3N)

echo "${teetimes:0:1000}"

echo "----------------------------------"

count=$(echo "$teetimes" | jq 'length')
echo "# of Tee Times: $count"

echo
echo "----------------------------------"
echo "Time: $((end - start)) ms"
