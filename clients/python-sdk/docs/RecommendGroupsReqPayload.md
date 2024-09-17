# RecommendGroupsReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**group_size** | **int** | The number of chunks to fetch for each group. This is the number of chunks which will be returned in the response for each group. The default is 3. If this is set to a large number, we recommend setting slim_chunks to true to avoid returning the content and chunk_html of the chunks so as to reduce latency due to content download and serialization. | [optional] 
**limit** | **int** | The number of groups to return. This is the number of groups which will be returned in the response. The default is 10. | [optional] 
**negative_group_ids** | **List[str]** | The ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups. | [optional] 
**negative_group_tracking_ids** | **List[str]** | The ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups. | [optional] 
**positive_group_ids** | **List[str]** | The ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups. | [optional] 
**positive_group_tracking_ids** | **List[str]** | The ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups. | [optional] 
**recommend_type** | [**RecommendType**](RecommendType.md) |  | [optional] 
**slim_chunks** | **bool** | Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false. | [optional] 
**strategy** | [**RecommendationStrategy**](RecommendationStrategy.md) |  | [optional] 
**user_id** | **str** | The user_id is the id of the user who is making the request. This is used to track user interactions with the rrecommendation results. | [optional] 

## Example

```python
from trieve_py_client.models.recommend_groups_req_payload import RecommendGroupsReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of RecommendGroupsReqPayload from a JSON string
recommend_groups_req_payload_instance = RecommendGroupsReqPayload.from_json(json)
# print the JSON string representation of the object
print(RecommendGroupsReqPayload.to_json())

# convert the object into a dict
recommend_groups_req_payload_dict = recommend_groups_req_payload_instance.to_dict()
# create an instance of RecommendGroupsReqPayload from a dict
recommend_groups_req_payload_form_dict = recommend_groups_req_payload.from_dict(recommend_groups_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


