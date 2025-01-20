# GenerateOffChunksReqPayload


## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**audio_input** | **str** | Audio input to be used in the chat. This will be used to generate the audio tokens for the model. The default is None. | [optional] 
**chunk_ids** | **List[str]** | The ids of the chunks to be retrieved and injected into the context window for RAG. | 
**context_options** | [**ContextOptions**](ContextOptions.md) |  | [optional] 
**frequency_penalty** | **float** | Frequency penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model&#39;s likelihood to repeat the same line verbatim. Default is 0.7. | [optional] 
**highlight_results** | **bool** | Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add &#x60;&lt;mark&gt;&lt;b&gt;&#x60; tags to the chunk_html of the chunks to highlight matching splits. | [optional] 
**image_config** | [**ImageConfig**](ImageConfig.md) |  | [optional] 
**image_urls** | **List[str]** | Image URLs to be used in the chat. These will be used to generate the image tokens for the model. The default is None. | [optional] 
**max_tokens** | **int** | The maximum number of tokens to generate in the chat completion. Default is None. | [optional] 
**presence_penalty** | **float** | Presence penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model&#39;s likelihood to talk about new topics. Default is 0.7. | [optional] 
**prev_messages** | [**List[ChatMessageProxy]**](ChatMessageProxy.md) | The previous messages to be placed into the chat history. There must be at least one previous message. | 
**prompt** | **str** | Prompt will be used to tell the model what to generate in the next message in the chat. The default is &#39;Respond to the previous instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:&#39;. You can also specify an empty string to leave the final message alone such that your user&#39;s final message can be used as the prompt. See docs.trieve.ai or contact us for more information. | [optional] 
**stop_tokens** | **List[str]** | Stop tokens are up to 4 sequences where the API will stop generating further tokens. Default is None. | [optional] 
**stream_response** | **bool** | Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true. | [optional] 
**temperature** | **float** | What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. Default is 0.5. | [optional] 
**user_id** | **str** | User ID is the id of the user who is making the request. This is used to track user interactions with the RAG results. | [optional] 

## Example

```python
from trieve_py_client.models.generate_off_chunks_req_payload import GenerateOffChunksReqPayload

# TODO update the JSON string below
json = "{}"
# create an instance of GenerateOffChunksReqPayload from a JSON string
generate_off_chunks_req_payload_instance = GenerateOffChunksReqPayload.from_json(json)
# print the JSON string representation of the object
print(GenerateOffChunksReqPayload.to_json())

# convert the object into a dict
generate_off_chunks_req_payload_dict = generate_off_chunks_req_payload_instance.to_dict()
# create an instance of GenerateOffChunksReqPayload from a dict
generate_off_chunks_req_payload_form_dict = generate_off_chunks_req_payload.from_dict(generate_off_chunks_req_payload_dict)
```
[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


