# RAGAnalyticsResponse


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**queries** | [**List[RagQueryEvent]**](RagQueryEvent.md) |  | 
**total_queries** | **int** |  | 
**usage_points** | [**List[UsageGraphPoint]**](UsageGraphPoint.md) |  | 
**created_at** | **str** |  | 
**dataset_id** | **str** |  | 
**detected_hallucinations** | **List[str]** |  | 
**hallucination_score** | **float** |  | 
**id** | **str** |  | 
**llm_response** | **str** |  | 
**query_rating** | [**SearchQueryRating**](SearchQueryRating.md) |  | [optional] 
**rag_type** | [**ClickhouseRagTypes**](ClickhouseRagTypes.md) |  | 
**results** | **List[object]** |  | 
**search_id** | **str** |  | 
**top_score** | **float** |  | 
**user_id** | **str** |  | 
**user_message** | **str** |  | 

## Example

```python
from trieve_py_client.models.rag_analytics_response import RAGAnalyticsResponse

# TODO update the JSON string below
json = "{}"
# create an instance of RAGAnalyticsResponse from a JSON string
rag_analytics_response_instance = RAGAnalyticsResponse.from_json(json)
# print the JSON string representation of the object
print(RAGAnalyticsResponse.to_json())

# convert the object into a dict
rag_analytics_response_dict = rag_analytics_response_instance.to_dict()
# create an instance of RAGAnalyticsResponse from a dict
rag_analytics_response_form_dict = rag_analytics_response.from_dict(rag_analytics_response_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


