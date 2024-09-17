# SearchCTRMetrics1


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filter** | [**SearchAnalyticsFilter**](SearchAnalyticsFilter.md) |  | [optional] 
**type** | **str** |  | 

## Example

```python
from trieve_py_client.models.search_ctr_metrics1 import SearchCTRMetrics1

# TODO update the JSON string below
json = "{}"
# create an instance of SearchCTRMetrics1 from a JSON string
search_ctr_metrics1_instance = SearchCTRMetrics1.from_json(json)
# print the JSON string representation of the object
print(SearchCTRMetrics1.to_json())

# convert the object into a dict
search_ctr_metrics1_dict = search_ctr_metrics1_instance.to_dict()
# create an instance of SearchCTRMetrics1 from a dict
search_ctr_metrics1_form_dict = search_ctr_metrics1.from_dict(search_ctr_metrics1_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


