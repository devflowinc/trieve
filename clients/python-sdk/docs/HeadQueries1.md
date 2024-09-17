# HeadQueries1


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.head_queries1 import HeadQueries1

# TODO update the JSON string below
json = "{}"
# create an instance of HeadQueries1 from a JSON string
head_queries1_instance = HeadQueries1.from_json(json)
# print the JSON string representation of the object
print(HeadQueries1.to_json())

# convert the object into a dict
head_queries1_dict = head_queries1_instance.to_dict()
# create an instance of HeadQueries1 from a dict
head_queries1_form_dict = head_queries1.from_dict(head_queries1_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


