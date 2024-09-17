# UpdateGroupByTrackingIDReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** | Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for. If not provided, the description will not be updated. | [optional] 
**metadata** | **object** | Optional metadata to assign to the chunk_group. This is a JSON object that can store any additional information you want to associate with the chunks inside of the chunk_group. | [optional] 
**name** | **str** | Name to assign to the chunk_group. Does not need to be unique. If not provided, the name will not be updated. | [optional] 
**tag_set** | **List[str]** | Optional tags to assign to the chunk_group. This is a list of strings that can be used to categorize the chunks inside the chunk_group. | [optional] 
**tracking_id** | **str** | Tracking Id of the chunk_group to update. | 

## Example

```python
from trieve_py_client.models.update_group_by_tracking_id_req_payload import UpdateGroupByTrackingIDReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateGroupByTrackingIDReqPayload from a JSON string
update_group_by_tracking_id_req_payload_instance = UpdateGroupByTrackingIDReqPayload.from_json(json)
# print the JSON string representation of the object
print(UpdateGroupByTrackingIDReqPayload.to_json())

# convert the object into a dict
update_group_by_tracking_id_req_payload_dict = update_group_by_tracking_id_req_payload_instance.to_dict()
# create an instance of UpdateGroupByTrackingIDReqPayload from a dict
update_group_by_tracking_id_req_payload_form_dict = update_group_by_tracking_id_req_payload.from_dict(update_group_by_tracking_id_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


