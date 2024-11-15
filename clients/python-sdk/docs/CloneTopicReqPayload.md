# CloneTopicReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**name** | **str** | The name of the topic. If this is not provided, the topic name is the same as the previous topic | [optional] 
**owner_id** | **str** | The owner_id of the topic. This is typically a browser fingerprint or your user&#39;s id. It is used to group topics together for a user. | 
**topic_id** | **str** | The topic_id to clone from | 

## Example

```python
from trieve_py_client.models.clone_topic_req_payload import CloneTopicReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CloneTopicReqPayload from a JSON string
clone_topic_req_payload_instance = CloneTopicReqPayload.from_json(json)
# print the JSON string representation of the object
print(CloneTopicReqPayload.to_json())

# convert the object into a dict
clone_topic_req_payload_dict = clone_topic_req_payload_instance.to_dict()
# create an instance of CloneTopicReqPayload from a dict
clone_topic_req_payload_form_dict = clone_topic_req_payload.from_dict(clone_topic_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


