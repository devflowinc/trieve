# GetGroupsForChunksReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_ids** | **List[str]** |  | [optional] 
**chunk_tracking_ids** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.get_groups_for_chunks_req_payload import GetGroupsForChunksReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of GetGroupsForChunksReqPayload from a JSON string
get_groups_for_chunks_req_payload_instance = GetGroupsForChunksReqPayload.from_json(json)
# print the JSON string representation of the object
print(GetGroupsForChunksReqPayload.to_json())

# convert the object into a dict
get_groups_for_chunks_req_payload_dict = get_groups_for_chunks_req_payload_instance.to_dict()
# create an instance of GetGroupsForChunksReqPayload from a dict
get_groups_for_chunks_req_payload_form_dict = get_groups_for_chunks_req_payload.from_dict(get_groups_for_chunks_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


