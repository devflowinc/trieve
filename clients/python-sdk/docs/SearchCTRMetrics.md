# SearchCTRMetrics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**avg_position_of_click** | **float** |  | 
**percent_searches_with_clicks** | **float** |  | 
**percent_searches_without_clicks** | **float** |  | 
**searches_with_clicks** | **int** |  | 

## Example

```python
from trieve_py_client.models.search_ctr_metrics import SearchCTRMetrics

# TODO update the JSON string below
json = "{}"
# create an instance of SearchCTRMetrics from a JSON string
search_ctr_metrics_instance = SearchCTRMetrics.from_json(json)
# print the JSON string representation of the object
print(SearchCTRMetrics.to_json())

# convert the object into a dict
search_ctr_metrics_dict = search_ctr_metrics_instance.to_dict()
# create an instance of SearchCTRMetrics from a dict
search_ctr_metrics_form_dict = search_ctr_metrics.from_dict(search_ctr_metrics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


