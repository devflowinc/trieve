# CreatePresignedUrlForCsvJsonlReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**description** | **str** | Description is an optional convience field so you do not have to remember what the file contains or is about. It will be included on the group resulting from the file which will hold its chunk. | [optional] 
**file_name** | **str** | Name of the file being uploaded, including the extension. Will be used to determine CSV or JSONL for processing. | 
**fulltext_boost_factor** | **float** | Amount to multiplicatevly increase the frequency of the tokens in the boost phrase for each row&#39;s chunk by. Applies to fulltext (SPLADE) and keyword (BM25) search. | [optional] 
**group_tracking_id** | **str** | Group tracking id is an optional field which allows you to specify the tracking id of the group that is created from the file. Chunks created will be created with the tracking id of &#x60;group_tracking_id|&lt;index of chunk&gt;&#x60; | [optional] 
**link** | **str** | Link to the file. This can also be any string. This can be used to filter when searching for the file&#39;s resulting chunks. The link value will not affect embedding creation. | [optional] 
**mappings** | [**List[ChunkReqPayloadMapping]**](ChunkReqPayloadMapping.md) | Specify all of the mappings between columns or fields in a CSV or JSONL file and keys in the ChunkReqPayload. Array fields like tag_set, image_urls, and group_tracking_ids can have multiple mappings. Boost phrase can also have multiple mappings which get concatenated. Other fields can only have one mapping and only the last mapping will be used. | [optional] 
**metadata** | **object** | Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. Will be passed down to the file&#39;s chunks. | [optional] 
**semantic_boost_factor** | **float** | Arbitrary float (positive or negative) specifying the multiplicate factor to apply before summing the phrase vector with the chunk_html embedding vector. Applies to semantic (embedding model) search. | [optional] 
**tag_set** | **List[str]** | Tag set is a comma separated list of tags which will be passed down to the chunks made from the file. Each tag will be joined with what&#39;s creatd per row of the CSV or JSONL file. | [optional] 
**time_stamp** | **str** | Time stamp should be an ISO 8601 combined date and time without timezone. Time_stamp is used for time window filtering and recency-biasing search results. Will be passed down to the file&#39;s chunks. | [optional] 
**upsert_by_tracking_id** | **bool** | Upsert by tracking_id. If true, chunks will be upserted by tracking_id. If false, chunks with the same tracking_id as another already existing chunk will be ignored. Defaults to true. | [optional] 

## Example

```python
from trieve_py_client.models.create_presigned_url_for_csv_jsonl_req_payload import CreatePresignedUrlForCsvJsonlReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of CreatePresignedUrlForCsvJsonlReqPayload from a JSON string
create_presigned_url_for_csv_jsonl_req_payload_instance = CreatePresignedUrlForCsvJsonlReqPayload.from_json(json)
# print the JSON string representation of the object
print(CreatePresignedUrlForCsvJsonlReqPayload.to_json())

# convert the object into a dict
create_presigned_url_for_csv_jsonl_req_payload_dict = create_presigned_url_for_csv_jsonl_req_payload_instance.to_dict()
# create an instance of CreatePresignedUrlForCsvJsonlReqPayload from a dict
create_presigned_url_for_csv_jsonl_req_payload_form_dict = create_presigned_url_for_csv_jsonl_req_payload.from_dict(create_presigned_url_for_csv_jsonl_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


