# LowConfidenceQueries


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**page** | **int** |  | [optional] 
**threshold** | **float** |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.low_confidence_queries import LowConfidenceQueries

# TODO update the JSON string below
json = "{}"
# create an instance of LowConfidenceQueries from a JSON string
low_confidence_queries_instance = LowConfidenceQueries.from_json(json)
# print the JSON string representation of the object
print(LowConfidenceQueries.to_json())

# convert the object into a dict
low_confidence_queries_dict = low_confidence_queries_instance.to_dict()
# create an instance of LowConfidenceQueries from a dict
low_confidence_queries_form_dict = low_confidence_queries.from_dict(low_confidence_queries_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


