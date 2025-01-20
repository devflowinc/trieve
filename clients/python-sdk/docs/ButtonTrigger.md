# ButtonTrigger


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**mode** | **str** |  | 
**remove_triggers** | **bool** |  | [optional] 
**selector** | **str** |  | 

## Example

```python
from trieve_py_client.models.button_trigger import ButtonTrigger

# TODO update the JSON string below
json = "{}"
# create an instance of ButtonTrigger from a JSON string
button_trigger_instance = ButtonTrigger.from_json(json)
# print the JSON string representation of the object
print(ButtonTrigger.to_json())

# convert the object into a dict
button_trigger_dict = button_trigger_instance.to_dict()
# create an instance of ButtonTrigger from a dict
button_trigger_form_dict = button_trigger.from_dict(button_trigger_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


