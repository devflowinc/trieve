# AddChunkToGroupReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_id** | **str** | Id of the chunk to make a member of the group. | [optional] 
**chunk_tracking_id** | **str** | Tracking Id of the chunk to make a member of the group. | [optional] 

## Example

```python
from trieve_py_client.models.add_chunk_to_group_req_payload import AddChunkToGroupReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of AddChunkToGroupReqPayload from a JSON string
add_chunk_to_group_req_payload_instance = AddChunkToGroupReqPayload.from_json(json)
# print the JSON string representation of the object
print(AddChunkToGroupReqPayload.to_json())

# convert the object into a dict
add_chunk_to_group_req_payload_dict = add_chunk_to_group_req_payload_instance.to_dict()
# create an instance of AddChunkToGroupReqPayload from a dict
add_chunk_to_group_req_payload_form_dict = add_chunk_to_group_req_payload.from_dict(add_chunk_to_group_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


