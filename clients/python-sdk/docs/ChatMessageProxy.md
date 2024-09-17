# ChatMessageProxy


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content** | **str** |  | 
**role** | [**RoleProxy**](RoleProxy.md) |  | 

## Example

```python
from trieve_py_client.models.chat_message_proxy import ChatMessageProxy

# TODO update the JSON string below
json = "{}"
# create an instance of ChatMessageProxy from a JSON string
chat_message_proxy_instance = ChatMessageProxy.from_json(json)
# print the JSON string representation of the object
print(ChatMessageProxy.to_json())

# convert the object into a dict
chat_message_proxy_dict = chat_message_proxy_instance.to_dict()
# create an instance of ChatMessageProxy from a dict
chat_message_proxy_form_dict = chat_message_proxy.from_dict(chat_message_proxy_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


