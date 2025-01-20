# RAG


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**detected_hallucinations** | **List[str]** | The detected hallucinations of the RAG event | [optional] 
**event_type** | **str** |  | 
**hallucination_score** | **float** | The hallucination score of the RAG event | [optional] 
**llm_response** | **str** | The response from the LLM | [optional] 
**query_rating** | [**SearchQueryRating**](SearchQueryRating.md) |  | [optional] 
**rag_type** | [**ClickhouseRagTypes**](ClickhouseRagTypes.md) |  | [optional] 
**results** | **List[object]** | The results of the RAG event | [optional] 
**search_id** | **str** | The search id to associate the RAG event with a search | [optional] 
**user_id** | **str** | The user id of the user who made the RAG event | [optional] 
**user_message** | **str** | The user message | 

## Example

```python
from trieve_py_client.models.rag import RAG

# TODO update the JSON string below
json = "{}"
# create an instance of RAG from a JSON string
rag_instance = RAG.from_json(json)
# print the JSON string representation of the object
print(RAG.to_json())

# convert the object into a dict
rag_dict = rag_instance.to_dict()
# create an instance of RAG from a dict
rag_form_dict = rag.from_dict(rag_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


