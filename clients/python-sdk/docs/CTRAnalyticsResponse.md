# CTRAnalyticsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**avg_position_of_click** | **float** |  | 
**percent_searches_with_clicks** | **float** |  | 
**percent_searches_without_clicks** | **float** |  | 
**searches_with_clicks** | **int** |  | 
**queries** | [**List[SearchQueriesWithClicksCTRResponse]**](SearchQueriesWithClicksCTRResponse.md) |  | 
**percent_recommendations_with_clicks** | **float** |  | 
**percent_recommendations_without_clicks** | **float** |  | 
**recommendations_with_clicks** | **int** |  | 
**recommendations** | [**List[RecommendationsWithClicksCTRResponse]**](RecommendationsWithClicksCTRResponse.md) |  | 

## Example

```python
from trieve_py_client.models.ctr_analytics_response import CTRAnalyticsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CTRAnalyticsResponse from a JSON string
ctr_analytics_response_instance = CTRAnalyticsResponse.from_json(json)
# print the JSON string representation of the object
print(CTRAnalyticsResponse.to_json())

# convert the object into a dict
ctr_analytics_response_dict = ctr_analytics_response_instance.to_dict()
# create an instance of CTRAnalyticsResponse from a dict
ctr_analytics_response_form_dict = ctr_analytics_response.from_dict(ctr_analytics_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


