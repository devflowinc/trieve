# SlimChunkMetadata


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**id** | **str** |  | 
**image_urls** | **List[Optional[str]]** |  | [optional] 
**link** | **str** |  | [optional] 
**location** | [**GeoInfo**](GeoInfo.md) |  | [optional] 
**metadata** | **object** |  | [optional] 
**num_value** | **float** |  | [optional] 
**tag_set** | **str** |  | [optional] 
**time_stamp** | **datetime** |  | [optional] 
**tracking_id** | **str** |  | [optional] 
**updated_at** | **datetime** |  | 
**weight** | **float** |  | 

## Example

```python
from trieve_py_client.models.slim_chunk_metadata import SlimChunkMetadata

# TODO update the JSON string below
json = "{}"
# create an instance of SlimChunkMetadata from a JSON string
slim_chunk_metadata_instance = SlimChunkMetadata.from_json(json)
# print the JSON string representation of the object
print(SlimChunkMetadata.to_json())

# convert the object into a dict
slim_chunk_metadata_dict = slim_chunk_metadata_instance.to_dict()
# create an instance of SlimChunkMetadata from a dict
slim_chunk_metadata_form_dict = slim_chunk_metadata.from_dict(slim_chunk_metadata_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


