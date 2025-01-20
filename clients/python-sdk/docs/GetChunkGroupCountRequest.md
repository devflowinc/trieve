# GetChunkGroupCountRequest


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**group_id** | **str** | The Id of the group to get the count for, is not required if group_tracking_id is provided. | [optional] 
**group_tracking_id** | **str** | The tracking id of the group to get the count for, is not required if group_id is provided. | [optional] 

## Example

```python
from trieve_py_client.models.get_chunk_group_count_request import GetChunkGroupCountRequest

# TODO update the JSON string below
json = "{}"
# create an instance of GetChunkGroupCountRequest from a JSON string
get_chunk_group_count_request_instance = GetChunkGroupCountRequest.from_json(json)
# print the JSON string representation of the object
print(GetChunkGroupCountRequest.to_json())

# convert the object into a dict
get_chunk_group_count_request_dict = get_chunk_group_count_request_instance.to_dict()
# create an instance of GetChunkGroupCountRequest from a dict
get_chunk_group_count_request_form_dict = get_chunk_group_count_request.from_dict(get_chunk_group_count_request_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


