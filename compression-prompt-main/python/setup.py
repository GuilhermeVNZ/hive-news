from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="compression_prompt",
    version="0.1.0",
    author="HiveLLM Team",
    author_email="team@hivellm.org",
    description="Fast statistical compression for LLM prompts - 50% token reduction with 91% quality retention",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/hivellm/compression-prompt",
    project_urls={
        "Homepage": "https://github.com/hivellm/compression-prompt",
        "Repository": "https://github.com/hivellm/compression-prompt",
        "Bug Reports": "https://github.com/hivellm/compression-prompt/issues",
        "Documentation": "https://github.com/hivellm/compression-prompt/blob/main/README.md",
    },
    packages=find_packages(exclude=["tests", "examples", "scripts"]),
    package_data={"compression_prompt": ["py.typed"]},
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Topic :: Text Processing",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.8",
    license="MIT",
    keywords=["llm", "compression", "prompt", "optimization", "token-reduction", "nlp", "ai"],
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-cov>=4.0.0",
            "black>=23.0.0",
            "mypy>=1.0.0",
            "ruff>=0.1.0",
            "build>=0.10.0",
            "twine>=4.0.0",
        ],
        "image": ["Pillow>=10.0.0"],
        "all": ["Pillow>=10.0.0"],
    },
    entry_points={
        "console_scripts": [
            "compress=compression_prompt.cli:main",
        ],
    },
)

