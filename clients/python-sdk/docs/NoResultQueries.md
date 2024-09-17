# NoResultQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.no_result_queries import NoResultQueries

# TODO update the JSON string below
json = "{}"
# create an instance of NoResultQueries from a JSON string
no_result_queries_instance = NoResultQueries.from_json(json)
# print the JSON string representation of the object
print(NoResultQueries.to_json())

# convert the object into a dict
no_result_queries_dict = no_result_queries_instance.to_dict()
# create an instance of NoResultQueries from a dict
no_result_queries_form_dict = no_result_queries.from_dict(no_result_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


