# CTRRecommendationsWithoutClicksResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**recommendations** | [**List[RecommendationsWithoutClicksCTRResponse]**](RecommendationsWithoutClicksCTRResponse.md) |  | 

## Example

```python
from trieve_py_client.models.ctr_recommendations_without_clicks_response import CTRRecommendationsWithoutClicksResponse

# TODO update the JSON string below
json = "{}"
# create an instance of CTRRecommendationsWithoutClicksResponse from a JSON string
ctr_recommendations_without_clicks_response_instance = CTRRecommendationsWithoutClicksResponse.from_json(json)
# print the JSON string representation of the object
print(CTRRecommendationsWithoutClicksResponse.to_json())

# convert the object into a dict
ctr_recommendations_without_clicks_response_dict = ctr_recommendations_without_clicks_response_instance.to_dict()
# create an instance of CTRRecommendationsWithoutClicksResponse from a dict
ctr_recommendations_without_clicks_response_form_dict = ctr_recommendations_without_clicks_response.from_dict(ctr_recommendations_without_clicks_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


