# CountQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.count_queries import CountQueries

# TODO update the JSON string below
json = "{}"
# create an instance of CountQueries from a JSON string
count_queries_instance = CountQueries.from_json(json)
# print the JSON string representation of the object
print(CountQueries.to_json())

# convert the object into a dict
count_queries_dict = count_queries_instance.to_dict()
# create an instance of CountQueries from a dict
count_queries_form_dict = count_queries.from_dict(count_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


