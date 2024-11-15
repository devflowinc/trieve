# FilterClicked


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**items** | **Dict[str, str]** | The filter items that were clicked in a hashmap ie. {filter_name: filter_value} where filter_name is filter_type::field_name | 
**request** | [**RequestInfo**](RequestInfo.md) |  | [optional] 
**user_id** | **str** | The user id of the user who clicked the items | [optional] 

## Example

```python
from trieve_py_client.models.filter_clicked import FilterClicked

# TODO update the JSON string below
json = "{}"
# create an instance of FilterClicked from a JSON string
filter_clicked_instance = FilterClicked.from_json(json)
# print the JSON string representation of the object
print(FilterClicked.to_json())

# convert the object into a dict
filter_clicked_dict = filter_clicked_instance.to_dict()
# create an instance of FilterClicked from a dict
filter_clicked_form_dict = filter_clicked.from_dict(filter_clicked_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


