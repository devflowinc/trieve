# CreateTopicReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**first_user_message** | **str** | The first message which will belong to the topic. The topic name is generated based on this message similar to how it works in the OpenAI chat UX if a name is not explicitly provided on the name request body key. | [optional] 
**name** | **str** | The name of the topic. If this is not provided, the topic name is generated from the first_user_message. | [optional] 
**owner_id** | **str** | The owner_id of the topic. This is typically a browser fingerprint or your user&#39;s id. It is used to group topics together for a user. | 

## Example

```python
from trieve_py_client.models.create_topic_req_payload import CreateTopicReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreateTopicReqPayload from a JSON string
create_topic_req_payload_instance = CreateTopicReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreateTopicReqPayload.to_json())

# convert the object into a dict
create_topic_req_payload_dict = create_topic_req_payload_instance.to_dict()
# create an instance of CreateTopicReqPayload from a dict
create_topic_req_payload_form_dict = create_topic_req_payload.from_dict(create_topic_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


