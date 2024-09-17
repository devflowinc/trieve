# DatasetAnalytics


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**avg_latency** | **float** |  | 
**p50** | **float** |  | 
**p95** | **float** |  | 
**p99** | **float** |  | 
**search_rps** | **float** |  | 
**total_queries** | **int** |  | 

## Example

```python
from trieve_py_client.models.dataset_analytics import DatasetAnalytics

# TODO update the JSON string below
json = "{}"
# create an instance of DatasetAnalytics from a JSON string
dataset_analytics_instance = DatasetAnalytics.from_json(json)
# print the JSON string representation of the object
print(DatasetAnalytics.to_json())

# convert the object into a dict
dataset_analytics_dict = dataset_analytics_instance.to_dict()
# create an instance of DatasetAnalytics from a dict
dataset_analytics_form_dict = dataset_analytics.from_dict(dataset_analytics_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


