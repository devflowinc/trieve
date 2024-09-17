# Message


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**completion_tokens** | **int** |  | [optional] 
**content** | **str** |  | 
**created_at** | **datetime** |  | 
**dataset_id** | **str** |  | 
**deleted** | **bool** |  | 
**id** | **str** |  | 
**prompt_tokens** | **int** |  | [optional] 
**role** | **str** |  | 
**sort_order** | **int** |  | 
**topic_id** | **str** |  | 
**updated_at** | **datetime** |  | 

## Example

```python
from trieve_py_client.models.message import Message

# TODO update the JSON string below
json = "{}"
# create an instance of Message from a JSON string
message_instance = Message.from_json(json)
# print the JSON string representation of the object
print(Message.to_json())

# convert the object into a dict
message_dict = message_instance.to_dict()
# create an instance of Message from a dict
message_form_dict = message.from_dict(message_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


