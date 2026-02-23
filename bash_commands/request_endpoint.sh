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
  --data-urlencode "date=2026-02-28" \
  --data-urlencode "players=4" \
  --data-urlencode 'coords={"min_lat":39.099016001081296,"min_lon":-94.41918493807773,"max_lat":39.15228348272485,"max_lon":-94.3324959427164}') #winterstone
  # --data-urlencode 'coords={"min_lat":38.950010842050155,"min_lon":-94.46220405145202,"max_lat":38.976705842620525,"max_lon":-94.41885955377136}') #teetering rocks
  # --data-urlencode 'coords={"min_lat":38.806081866102396,"min_lon":-94.85117254427148,"max_lat":38.859570093917206,"max_lon":-94.65616522005273}') #heritage
  # --data-urlencode 'coords={"min_lat":38.895206560610035,"min_lon":-94.55215176681716,"max_lat":38.94862777433994,"max_lon":-94.35714444259841}') #fred arbanas
  # --data-urlencode 'coords={"min_lat":38.686,"min_lon":-96.563,"max_lat":39.539,"max_lon":-93.528}')
  # --data-urlencode 'coords={min_lat:39.1618,min_lon:-94.899,max_lat:39.1752,max_lon:-94.8515}')
end=$(date +%s%3N)

echo "${teetimes:0:1000}"

echo "----------------------------------"

count=$(echo "$teetimes" | jq 'length')
echo "# of Tee Times: $count"

echo
echo "----------------------------------"
echo "Time: $((end - start)) ms"
