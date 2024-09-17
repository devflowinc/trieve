# TypoRange

The TypoRange struct is used to specify the range of which the query will be corrected if it has a typo.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**max** | **int** | The maximum number of characters that the query will be corrected if it has a typo. If not specified, this defaults to 8. | [optional] 
**min** | **int** | The minimum number of characters that the query will be corrected if it has a typo. If not specified, this defaults to 5. | 

## Example

```python
from trieve_py_client.models.typo_range import TypoRange

# TODO update the JSON string below
json = "{}"
# create an instance of TypoRange from a JSON string
typo_range_instance = TypoRange.from_json(json)
# print the JSON string representation of the object
print(TypoRange.to_json())

# convert the object into a dict
typo_range_dict = typo_range_instance.to_dict()
# create an instance of TypoRange from a dict
typo_range_form_dict = typo_range.from_dict(typo_range_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


