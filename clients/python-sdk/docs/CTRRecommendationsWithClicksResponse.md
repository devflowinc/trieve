# CTRRecommendationsWithClicksResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**recommendations** | [**List[RecommendationsWithClicksCTRResponse]**](RecommendationsWithClicksCTRResponse.md) |  | 

## Example

```python
from trieve_py_client.models.ctr_recommendations_with_clicks_response import CTRRecommendationsWithClicksResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CTRRecommendationsWithClicksResponse from a JSON string
ctr_recommendations_with_clicks_response_instance = CTRRecommendationsWithClicksResponse.from_json(json)
# print the JSON string representation of the object
print(CTRRecommendationsWithClicksResponse.to_json())

# convert the object into a dict
ctr_recommendations_with_clicks_response_dict = ctr_recommendations_with_clicks_response_instance.to_dict()
# create an instance of CTRRecommendationsWithClicksResponse from a dict
ctr_recommendations_with_clicks_response_form_dict = ctr_recommendations_with_clicks_response.from_dict(ctr_recommendations_with_clicks_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


