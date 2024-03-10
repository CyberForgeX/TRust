import asyncio
import os
import logging
import aiofiles
import coloredlogs
from openai import AsyncOpenAI
from prompt_toolkit import PromptSession

# Setup

coloredlogs.install()
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger('async_code_process')
openai_api_key = os.getenv('OPENAI_API_KEY')
async_client = AsyncOpenAI(api_key=openai_api_key)
PromptSession().prompt_async()

async def list_rust_files(directory):
    return [entry.path for entry in os.scandir(directory) if entry.is_file() and entry.name.endswith('.rs')]

async def async_fix_pov_with_gpt4(content):
    GPT_MODEL = "gpt-4-0125-preview"
    messages = [
        {"role": "system", "content": "Your task is to fix code errors in the folder provided and make all the files logical. Get overview first, then make folder ./src and code all files."},
        {"role": "user", "content": content},
    ]
    try:
        response = await async_client.chat.completions.create(model=GPT_MODEL, messages=messages, temperature=0)
        improved_code = response.choices[0].message.content
        return {"status": "success", "content": improved_code}
    except Exception as e:
        logger.error(f"An error occurred: {e}")
        return {"status": "error", "error": str(e)}

async def handle_feedback_and_improvement(filepath, improved_code):
    logger.info(f"AI's suggestion for {filepath}:\n{improved_code}")
    user_feedback = await session.prompt_async("Do you accept these changes? (yes/no/guide): ")
    if user_feedback.lower() == 'yes':
        await write_file_content(filepath, improved_code)
        logger.info(f"Improved code written back to {filepath}.")
    elif user_feedback.lower() == 'guide':
        guidance = await session.prompt_async("Provide guidance to the AI: ")
        await async_fix_pov_with_gpt4(guidance)
    else:
        logger.info("No changes made to the original code.")

async def read_file_content(filepath):
    async with aiofiles.open(filepath, 'r', encoding='utf-8') as file:
        return await file.read()

async def write_file_content(filepath, content):
    async with aiofiles.open(filepath, 'w', encoding='utf-8') as file:
        await file.write(content)

async def process_file(filepath):
    content = await read_file_content(filepath)
    improvement_result = await async_fix_pov_with_gpt4(content)
    if improvement_result["status"] == "success":
        await handle_feedback_and_improvement(filepath, improvement_result["content"])
    else:
        logger.error(f"Error improving {filepath}: {improvement_result['error']}")

async def process_directory(directory):
    rust_files = await list_rust_files(directory)
    tasks = [process_file(filepath) for filepath in rust_files]
    await asyncio.gather(*tasks)

if __name__ == "__main__":
    directory = "./src2"
    asyncio.run(process_directory(directory))
