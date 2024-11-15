# Click


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**clicked_items** | [**ChunkWithPosition**](ChunkWithPosition.md) |  | 
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**request** | [**RequestInfo**](RequestInfo.md) |  | [optional] 
**user_id** | **str** | The user id of the user who clicked the items | [optional] 

## Example

```python
from trieve_py_client.models.click import Click

# TODO update the JSON string below
json = "{}"
# create an instance of Click from a JSON string
click_instance = Click.from_json(json)
# print the JSON string representation of the object
print(Click.to_json())

# convert the object into a dict
click_dict = click_instance.to_dict()
# create an instance of Click from a dict
click_form_dict = click.from_dict(click_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


