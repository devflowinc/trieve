# EventTypes


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**event_name** | **str** | The name of the event | 
**event_type** | **str** |  | 
**items** | **Dict[str, str]** | The filter items that were clicked in a hashmap ie. {filter_name: filter_value} where filter_name is filter_type::field_name | 
**metadata** | **object** | Any other metadata associated with the event | [optional] 
**request** | [**RequestInfo**](RequestInfo.md) |  | [optional] 
**user_id** | **str** | The user id of the user who made the recommendation | [optional] 
**is_conversion** | **bool** | Whether the event is a conversion event | [optional] 
**clicked_items** | [**ChunkWithPosition**](ChunkWithPosition.md) |  | 
**currency** | **str** | The currency of the purchase | [optional] 
**value** | **float** | The value of the purchase | [optional] 
**latency** | **float** | Latency of the search | [optional] 
**query** | **str** | The search query | 
**query_rating** | [**SearchQueryRating**](SearchQueryRating.md) |  | [optional] 
**request_params** | **object** | The request params of the recommendation | [optional] 
**results** | **List[object]** | The results of the Recommendation event | [optional] 
**search_type** | [**ClickhouseSearchTypes**](ClickhouseSearchTypes.md) |  | [optional] 
**top_score** | **float** | Top score of the recommendation | [optional] 
**detected_hallucinations** | **List[str]** | The detected hallucinations of the RAG event | [optional] 
**hallucination_score** | **float** | The hallucination score of the RAG event | [optional] 
**llm_response** | **str** | The response from the LLM | [optional] 
**rag_type** | [**ClickhouseRagTypes**](ClickhouseRagTypes.md) |  | [optional] 
**search_id** | **str** | The search id to associate the RAG event with a search | [optional] 
**user_message** | **str** | The user message | 
**negative_ids** | **List[str]** | Negative ids used for the recommendation | [optional] 
**negative_tracking_ids** | **List[str]** | Negative tracking ids used for the recommendation | [optional] 
**positive_ids** | **List[str]** | Positive ids used for the recommendation | [optional] 
**positive_tracking_ids** | **List[str]** | Positive tracking ids used for the recommendation | [optional] 
**recommendation_type** | [**ClickhouseRecommendationTypes**](ClickhouseRecommendationTypes.md) |  | [optional] 

## Example

```python
from trieve_py_client.models.event_types import EventTypes

# TODO update the JSON string below
json = "{}"
# create an instance of EventTypes from a JSON string
event_types_instance = EventTypes.from_json(json)
# print the JSON string representation of the object
print(EventTypes.to_json())

# convert the object into a dict
event_types_dict = event_types_instance.to_dict()
# create an instance of EventTypes from a dict
event_types_form_dict = event_types.from_dict(event_types_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


