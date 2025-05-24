find "$UPLOAD_PATH" -type f -print0 | xargs -0 -I{} -P 4 sh -c '
    file="{}"
    # Get the relative path of the file
    relative_path="${file#$UPLOAD_PATH/}"
    # Construct the URL for the BunnyCDN storage
    upload_url="https://storage.bunnycdn.com/$STORAGE_ZONE/$relative_path"
    # Upload the file to BunnyCDN
    echo -n "Uploading $file ... "
    curl --request PUT --url "$upload_url" \
      --header "AccessKey: $BUNNY_API_KEY" \
      --header "Content-Type: application/octet-stream" \
      --header "accept: application/json" \
      --data-binary "@$file" \
      -s \
      -w " \n"
  '
echo "Done!"
