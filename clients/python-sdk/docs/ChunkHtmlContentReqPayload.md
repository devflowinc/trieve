# ChunkHtmlContentReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**body_remove_strings** | **List[str]** | Text strings to remove from body when creating chunks for each page | [optional] 
**chunk_html** | **str** | The HTML content to be split into chunks | 
**heading_remove_strings** | **List[str]** | Text strings to remove from headings when creating chunks for each page | [optional] 

## Example

```python
from trieve_py_client.models.chunk_html_content_req_payload import ChunkHtmlContentReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkHtmlContentReqPayload from a JSON string
chunk_html_content_req_payload_instance = ChunkHtmlContentReqPayload.from_json(json)
# print the JSON string representation of the object
print(ChunkHtmlContentReqPayload.to_json())

# convert the object into a dict
chunk_html_content_req_payload_dict = chunk_html_content_req_payload_instance.to_dict()
# create an instance of ChunkHtmlContentReqPayload from a dict
chunk_html_content_req_payload_form_dict = chunk_html_content_req_payload.from_dict(chunk_html_content_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


