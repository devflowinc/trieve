# ChunkReturnTypes


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_html** | **str** |  | [optional] 
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
from trieve_py_client.models.chunk_return_types import ChunkReturnTypes

# TODO update the JSON string below
json = "{}"
# create an instance of ChunkReturnTypes from a JSON string
chunk_return_types_instance = ChunkReturnTypes.from_json(json)
# print the JSON string representation of the object
print(ChunkReturnTypes.to_json())

# convert the object into a dict
chunk_return_types_dict = chunk_return_types_instance.to_dict()
# create an instance of ChunkReturnTypes from a dict
chunk_return_types_form_dict = chunk_return_types.from_dict(chunk_return_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


