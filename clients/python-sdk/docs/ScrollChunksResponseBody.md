# ScrollChunksResponseBody


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunks** | [**List[ChunkMetadata]**](ChunkMetadata.md) |  | 

## Example

```python
from trieve_py_client.models.scroll_chunks_response_body import ScrollChunksResponseBody

# TODO update the JSON string below
json = "{}"
# create an instance of ScrollChunksResponseBody from a JSON string
scroll_chunks_response_body_instance = ScrollChunksResponseBody.from_json(json)
# print the JSON string representation of the object
print(ScrollChunksResponseBody.to_json())

# convert the object into a dict
scroll_chunks_response_body_dict = scroll_chunks_response_body_instance.to_dict()
# create an instance of ScrollChunksResponseBody from a dict
scroll_chunks_response_body_form_dict = scroll_chunks_response_body.from_dict(scroll_chunks_response_body_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


