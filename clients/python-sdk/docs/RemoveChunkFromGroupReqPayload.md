# RemoveChunkFromGroupReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_id** | **str** | Id of the chunk to remove from the group. | 

## Example

```python
from trieve_py_client.models.remove_chunk_from_group_req_payload import RemoveChunkFromGroupReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of RemoveChunkFromGroupReqPayload from a JSON string
remove_chunk_from_group_req_payload_instance = RemoveChunkFromGroupReqPayload.from_json(json)
# print the JSON string representation of the object
print(RemoveChunkFromGroupReqPayload.to_json())

# convert the object into a dict
remove_chunk_from_group_req_payload_dict = remove_chunk_from_group_req_payload_instance.to_dict()
# create an instance of RemoveChunkFromGroupReqPayload from a dict
remove_chunk_from_group_req_payload_form_dict = remove_chunk_from_group_req_payload.from_dict(remove_chunk_from_group_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


