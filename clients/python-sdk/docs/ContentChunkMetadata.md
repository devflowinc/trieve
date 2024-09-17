# ContentChunkMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_html** | **str** |  | [optional] 
**id** | **str** |  | 
**image_urls** | **List[Optional[str]]** |  | [optional] 
**num_value** | **float** |  | [optional] 
**time_stamp** | **datetime** |  | [optional] 
**tracking_id** | **str** |  | [optional] 
**weight** | **float** |  | 

## Example

```python
from trieve_py_client.models.content_chunk_metadata import ContentChunkMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of ContentChunkMetadata from a JSON string
content_chunk_metadata_instance = ContentChunkMetadata.from_json(json)
# print the JSON string representation of the object
print(ContentChunkMetadata.to_json())

# convert the object into a dict
content_chunk_metadata_dict = content_chunk_metadata_instance.to_dict()
# create an instance of ContentChunkMetadata from a dict
content_chunk_metadata_form_dict = content_chunk_metadata.from_dict(content_chunk_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


