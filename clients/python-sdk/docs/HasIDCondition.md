# HasIDCondition


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**ids** | **List[str]** |  | [optional] 
**tracking_ids** | **List[str]** |  | [optional] 

## Example

```python
from trieve_py_client.models.has_id_condition import HasIDCondition

# TODO update the JSON string below
json = "{}"
# create an instance of HasIDCondition from a JSON string
has_id_condition_instance = HasIDCondition.from_json(json)
# print the JSON string representation of the object
print(HasIDCondition.to_json())

# convert the object into a dict
has_id_condition_dict = has_id_condition_instance.to_dict()
# create an instance of HasIDCondition from a dict
has_id_condition_form_dict = has_id_condition.from_dict(has_id_condition_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


