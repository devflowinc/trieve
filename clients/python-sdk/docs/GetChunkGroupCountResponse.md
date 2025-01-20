# GetChunkGroupCountResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**count** | **int** | The count of chunks in the given group. | 
**group_id** | **str** | The Id of the group to get the count for. | 

## Example

```python
from trieve_py_client.models.get_chunk_group_count_response import GetChunkGroupCountResponse

# TODO update the JSON string below
json = "{}"
# create an instance of GetChunkGroupCountResponse from a JSON string
get_chunk_group_count_response_instance = GetChunkGroupCountResponse.from_json(json)
# print the JSON string representation of the object
print(GetChunkGroupCountResponse.to_json())

# convert the object into a dict
get_chunk_group_count_response_dict = get_chunk_group_count_response_instance.to_dict()
# create an instance of GetChunkGroupCountResponse from a dict
get_chunk_group_count_response_form_dict = get_chunk_group_count_response.from_dict(get_chunk_group_count_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


