# UpdateChunkByTrackingIdData


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**chunk_html** | **str** | HTML content of the chunk you want to update. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content. If no chunk_html is provided, the existing chunk_html will be used. | [optional] 
**convert_html_to_text** | **bool** | Convert HTML to raw text before processing to avoid adding noise to the vector embeddings. By default this is true. If you are using HTML content that you want to be included in the vector embeddings, set this to false. | [optional] 
**group_ids** | **List[str]** | Group ids are the ids of the groups that the chunk should be placed into. This is useful for when you want to update a chunk and add it to a group or multiple groups in one request. | [optional] 
**group_tracking_ids** | **List[str]** | Group tracking_ids are the tracking_ids of the groups that the chunk should be placed into. This is useful for when you want to update a chunk and add it to a group or multiple groups in one request. | [optional] 
**link** | **str** | Link of the chunk you want to update. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation. If no link is provided, the existing link will be used. | [optional] 
**metadata** | **object** | The metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. If no metadata is provided, the existing metadata will be used. | [optional] 
**time_stamp** | **str** | Time_stamp should be an ISO 8601 combined date and time without timezone. It is used for time window filtering and recency-biasing search results. If no time_stamp is provided, the existing time_stamp will be used. | [optional] 
**tracking_id** | **str** | Tracking_id of the chunk you want to update. This is required to match an existing chunk. | 
**weight** | **float** | Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk&#39;s dataset dataset. If no weight is provided, the existing weight will be used. | [optional] 

## Example

```python
from trieve_py_client.models.update_chunk_by_tracking_id_data import UpdateChunkByTrackingIdData

# TODO update the JSON string below
json = "{}"
# create an instance of UpdateChunkByTrackingIdData from a JSON string
update_chunk_by_tracking_id_data_instance = UpdateChunkByTrackingIdData.from_json(json)
# print the JSON string representation of the object
print(UpdateChunkByTrackingIdData.to_json())

# convert the object into a dict
update_chunk_by_tracking_id_data_dict = update_chunk_by_tracking_id_data_instance.to_dict()
# create an instance of UpdateChunkByTrackingIdData from a dict
update_chunk_by_tracking_id_data_form_dict = update_chunk_by_tracking_id_data.from_dict(update_chunk_by_tracking_id_data_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


