# SearchWithinGroupReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**content_only** | **bool** | Set content_only to true to only returning the chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false. | [optional] 
**filters** | [**ChunkFilter**](ChunkFilter.md) |  | [optional] 
**get_total_pages** | **bool** | Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms). | [optional] 
**group_id** | **str** | Group specifies the group to search within. Results will only consist of chunks which are bookmarks within the specified group. | [optional] 
**group_tracking_id** | **str** | Group_tracking_id specifies the group to search within by tracking id. Results will only consist of chunks which are bookmarks within the specified group. If both group_id and group_tracking_id are provided, group_id will be used. | [optional] 
**highlight_options** | [**HighlightOptions**](HighlightOptions.md) |  | [optional] 
**page** | **int** | The page of chunks to fetch. Page is 1-indexed. | [optional] 
**page_size** | **int** | The page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time. | [optional] 
**query** | [**QueryTypes**](QueryTypes.md) |  | 
**remove_stop_words** | **bool** | If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved. | [optional] 
**score_threshold** | **float** | Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0. | [optional] 
**search_type** | [**SearchMethod**](SearchMethod.md) |  | 
**slim_chunks** | **bool** | Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false. | [optional] 
**sort_options** | [**SortOptions**](SortOptions.md) |  | [optional] 
**typo_options** | [**TypoOptions**](TypoOptions.md) |  | [optional] 
**use_quote_negated_terms** | **bool** | If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false. | [optional] 
**user_id** | **str** | The user_id is the id of the user who is making the request. This is used to track user interactions with the search results. | [optional] 

## Example

```python
from trieve_py_client.models.search_within_group_req_payload import SearchWithinGroupReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of SearchWithinGroupReqPayload from a JSON string
search_within_group_req_payload_instance = SearchWithinGroupReqPayload.from_json(json)
# print the JSON string representation of the object
print(SearchWithinGroupReqPayload.to_json())

# convert the object into a dict
search_within_group_req_payload_dict = search_within_group_req_payload_instance.to_dict()
# create an instance of SearchWithinGroupReqPayload from a dict
search_within_group_req_payload_form_dict = search_within_group_req_payload.from_dict(search_within_group_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


